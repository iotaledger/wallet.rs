// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use iota_client::{
    api_types::dto::LedgerInclusionStateDto,
    block::{input::Input, output::OutputId, payload::transaction::TransactionEssence, BlockId},
    Error as ClientError,
};

use crate::account::{
    handle::AccountHandle,
    types::{InclusionState, Transaction},
    Account,
};

// ignore outputs and transactions from other networks
// check if outputs are unspent, rebroadcast, reattach...
// also revalidate that the locked outputs needs to be there, maybe there was a conflict or the transaction got
// confirmed, then they should get removed

impl AccountHandle {
    /// Sync transactions and reattach them if unconfirmed. Returns the transaction with updated metadata and spent
    /// output ids that don't need to be locked anymore
    pub(crate) async fn sync_pending_transactions(&self) -> crate::Result<()> {
        log::debug!("[SYNC] sync pending transactions");
        let account = self.read().await;

        if account.pending_transactions.is_empty() {
            return Ok(());
        }

        let network_id = self.client.get_network_id()?;

        let mut updated_transactions = Vec::new();
        let mut spent_output_ids = Vec::new();
        // Inputs from conflicting transactions that are unspent, but should be removed from the locked outputs so they
        // are available again
        let mut output_ids_to_unlock = Vec::new();
        let mut transactions_to_reattach = Vec::new();

        for transaction_id in &account.pending_transactions {
            log::debug!("[SYNC] sync pending transaction {}", transaction_id);
            let transaction = account
                .transactions
                .get(transaction_id)
                // panic during development to easier detect if something is wrong, should be handled different later
                .expect("transaction id stored, but transaction is missing")
                .clone();

            // only check transaction from the network we're connected to
            if transaction.network_id != network_id {
                continue;
            }

            // check if we have an output (remainder, if not sending to an own address) that got created by this
            // transaction, if that's the case, then the transaction got confirmed
            let transaction_output = account
                .outputs
                .keys()
                .into_iter()
                .find(|o| o.transaction_id() == transaction_id);

            if let Some(transaction_output) = transaction_output {
                // Save to unwrap, we just got the output
                let confirmed_output_data = account.outputs.get(transaction_output).expect("output exists");
                log::debug!(
                    "[SYNC] confirmed transaction {} in block {}",
                    transaction_id,
                    confirmed_output_data.metadata.block_id
                );
                updated_transaction_and_outputs(
                    transaction,
                    Some(BlockId::from_str(&confirmed_output_data.metadata.block_id)?),
                    InclusionState::Confirmed,
                    &mut updated_transactions,
                    &mut spent_output_ids,
                );
                continue;
            }

            // Check if the inputs of the transaction are still unspent
            let TransactionEssence::Regular(essence) = transaction.payload.essence();
            let mut input_got_spent = false;
            for input in essence.inputs() {
                if let Input::Utxo(input) = input {
                    if let Some(input) = account.outputs.get(input.output_id()) {
                        if input.is_spent {
                            input_got_spent = true;
                        }
                    }
                }
            }

            if let Some(block_id) = transaction.block_id {
                match self.client.get_block_metadata(&block_id).await {
                    Ok(metadata) => {
                        if let Some(inclusion_state) = metadata.ledger_inclusion_state {
                            match inclusion_state {
                                LedgerInclusionStateDto::Included => {
                                    log::debug!(
                                        "[SYNC] confirmed transaction {} in block {}",
                                        transaction_id,
                                        metadata.block_id
                                    );
                                    updated_transaction_and_outputs(
                                        transaction,
                                        Some(BlockId::from_str(&metadata.block_id)?),
                                        InclusionState::Confirmed,
                                        &mut updated_transactions,
                                        &mut spent_output_ids,
                                    );
                                }
                                LedgerInclusionStateDto::Conflicting => {
                                    log::debug!("[SYNC] conflicting transaction {}", transaction_id);
                                    // try to get the included block, because maybe only this attachment is
                                    // conflicting because it got confirmed in another block
                                    if let Ok(included_block) =
                                        self.client.get_included_block(&transaction.payload.id()).await
                                    {
                                        updated_transaction_and_outputs(
                                            transaction,
                                            Some(included_block.id()),
                                            // block metadata was Conflicting, but it's confirmed in another attachment
                                            InclusionState::Confirmed,
                                            &mut updated_transactions,
                                            &mut spent_output_ids,
                                        );
                                    } else {
                                        updated_transaction_and_outputs(
                                            transaction,
                                            None,
                                            InclusionState::Conflicting,
                                            &mut updated_transactions,
                                            &mut spent_output_ids,
                                        );
                                    }
                                }
                                LedgerInclusionStateDto::NoTransaction => {
                                    unreachable!(
                                        "We should only get the metadata for blocks with a transaction payload"
                                    )
                                }
                            }
                        } else {
                            // no need to reattach if one input got spent
                            if input_got_spent {
                                process_transaction_with_unknown_state(
                                    &account,
                                    transaction,
                                    &mut updated_transactions,
                                    &mut output_ids_to_unlock,
                                )?;
                            } else {
                                let time_now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("time went backwards")
                                    .as_millis();
                                // Reattach if older than 30 seconds
                                if transaction.timestamp + 30000 < time_now {
                                    // only reattach if inputs are still unspent
                                    transactions_to_reattach.push(transaction);
                                }
                            }
                        }
                    }
                    Err(ClientError::NotFound(_)) => {
                        // no need to reattach if one input got spent
                        if input_got_spent {
                            process_transaction_with_unknown_state(
                                &account,
                                transaction,
                                &mut updated_transactions,
                                &mut output_ids_to_unlock,
                            )?;
                        } else {
                            let time_now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("time went backwards")
                                .as_millis();
                            // Reattach if older than 30 seconds
                            if transaction.timestamp + 30000 < time_now {
                                // only reattach if inputs are still unspent
                                transactions_to_reattach.push(transaction);
                            }
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            } else {
                // transaction wasn't submitted yet, so we have to send it again
                // no need to reattach if one input got spent
                if input_got_spent {
                } else {
                    // only reattach if inputs are still unspent
                    transactions_to_reattach.push(transaction);
                }
            }
        }
        drop(account);

        for mut transaction in transactions_to_reattach {
            log::debug!("[SYNC] reattach transaction");
            let reattached_block = self.submit_transaction_payload(transaction.payload.clone()).await?;
            transaction.block_id.replace(reattached_block);
            updated_transactions.push(transaction);
        }

        // updates account with balances, output ids, outputs
        self.update_account_with_transactions(updated_transactions, spent_output_ids, output_ids_to_unlock)
            .await?;

        Ok(())
    }
}

// Set the outputs as spent so they will not be used as input again
fn updated_transaction_and_outputs(
    mut transaction: Transaction,
    block_id: Option<BlockId>,
    inclusion_state: InclusionState,
    updated_transactions: &mut Vec<Transaction>,
    spent_output_ids: &mut Vec<OutputId>,
) {
    transaction.block_id = block_id;
    transaction.inclusion_state = inclusion_state;
    // get spent inputs
    let TransactionEssence::Regular(essence) = transaction.payload.essence();
    for input in essence.inputs() {
        if let Input::Utxo(input) = input {
            spent_output_ids.push(*input.output_id());
        }
    }
    updated_transactions.push(transaction);
}

// When a transaction got pruned, the inputs and outputs are also not available, then this could mean that it was
// confirmed and the created outputs got also already spent and pruned or the inputs got spent in another transaction
fn process_transaction_with_unknown_state(
    account: &Account,
    mut transaction: Transaction,
    updated_transactions: &mut Vec<Transaction>,
    output_ids_to_unlock: &mut Vec<OutputId>,
) -> crate::Result<()> {
    let mut all_inputs_spent = true;
    let TransactionEssence::Regular(essence) = transaction.payload.essence();
    for input in essence.inputs() {
        if let Input::Utxo(input) = input {
            if let Some(output_data) = account.outputs.get(input.output_id()) {
                if !output_data.metadata.is_spent {
                    // unspent output needs to be made available again
                    output_ids_to_unlock.push(*input.output_id());
                    all_inputs_spent = false;
                }
            } else {
                all_inputs_spent = false;
            }
        }
    }
    // If only a part of the inputs got spent, then it couldn't happen with this transaction, so it's conflicting
    if all_inputs_spent {
        transaction.inclusion_state = InclusionState::UnknownPruned;
    } else {
        transaction.inclusion_state = InclusionState::Conflicting;
    }
    updated_transactions.push(transaction);
    Ok(())
}

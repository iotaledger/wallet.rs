// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    handle::AccountHandle,
    operations::transfer::submit_transaction::submit_transaction_payload,
    types::{InclusionState, Transaction},
};

use iota_client::{
    bee_message::{
        input::{Input, UtxoInput},
        output::OutputId,
        payload::transaction::Essence,
        MessageId,
    },
    bee_rest_api::types::dtos::LedgerInclusionStateDto,
};

use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

// ignore outputs and transactions from other networks
// check if outputs are unspent, rebroadcast, reattach...
// also revalidate that the locked outputs needs to be there, maybe there was a conflict or the transaction got
// confirmed, then they should get removed sync_transactions(){
// retry(message_id, sync: false)
// }.await?;

/// Sync transactions and reattach them if unconfirmed. Returns the transaction with updated metadata and spent output
/// ids that don't need to be locked anymore
pub(crate) async fn sync_transactions(
    account_handle: &AccountHandle,
) -> crate::Result<(Vec<Transaction>, Vec<OutputId>)> {
    log::debug!("[SYNC] sync pending transactions");
    let account = account_handle.read().await;
    let client = crate::client::get_client().await?;
    let network_id = client.get_network_id().await?;

    let mut updated_transactions = Vec::new();
    let mut spent_output_ids = Vec::new();
    let mut transactions_to_reattach = Vec::new();

    for transaction_id in &account.pending_transactions {
        let transaction = account
            .transactions
            .get(transaction_id)
            // panic during development to easier detect if something is wrong, should be handled different later
            .expect("transaction id stored, but transaction is missing")
            .clone();
        // only check transaction from the network we're connected to
        if transaction.network_id == network_id {
            // use first output of the transaction to check if it got confirmed
            if let Ok(output_response) = client
                .get_output(&UtxoInput::from(OutputId::new(transaction.payload.id(), 0)?))
                .await
            {
                updated_transaction_and_outputs(
                    transaction,
                    MessageId::from_str(&output_response.message_id)?,
                    InclusionState::Confirmed,
                    &mut updated_transactions,
                    &mut spent_output_ids,
                );
            } else if let Some(message_id) = transaction.message_id {
                let metadata = client.get_message().metadata(&message_id).await?;
                if let Some(inclusion_state) = metadata.ledger_inclusion_state {
                    match inclusion_state {
                        LedgerInclusionStateDto::Included => {
                            updated_transaction_and_outputs(
                                transaction,
                                MessageId::from_str(&metadata.message_id)?,
                                InclusionState::Confirmed,
                                &mut updated_transactions,
                                &mut spent_output_ids,
                            );
                        }
                        LedgerInclusionStateDto::Conflicting => {
                            // try to get the included message, because maybe only this attachment is conflicting
                            // because it got confirmed in another message
                            if let Ok(included_message) = client.get_included_message(&transaction.payload.id()).await {
                                updated_transaction_and_outputs(
                                    transaction,
                                    included_message.id().0,
                                    InclusionState::Confirmed,
                                    &mut updated_transactions,
                                    &mut spent_output_ids,
                                );
                            } else {
                                // a part of the outputs could still be unspent, but when we set it as spent we will get
                                // it as unspent later during syncing again
                                updated_transaction_and_outputs(
                                    transaction,
                                    MessageId::from_str(&metadata.message_id)?,
                                    InclusionState::Conflicting,
                                    &mut updated_transactions,
                                    &mut spent_output_ids,
                                );
                            }
                        }
                        LedgerInclusionStateDto::NoTransaction => {}
                    }
                } else {
                    let time_now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();
                    // Reattach if older than 30 seconds
                    if transaction.timestamp + 30000 < time_now {
                        transactions_to_reattach.push(transaction);
                    }
                }
            } else {
                // transaction wasn't submitted yet, so we have to send it again
                transactions_to_reattach.push(transaction);
            }
        }
    }
    drop(account);
    for mut transaction in transactions_to_reattach {
        log::debug!("[SYNC] reattach transaction");
        let reattached_msg = submit_transaction_payload(account_handle, transaction.payload.clone()).await?;
        transaction.message_id.replace(reattached_msg);
        updated_transactions.push(transaction);
    }

    Ok((updated_transactions, spent_output_ids))
}

fn updated_transaction_and_outputs(
    mut transaction: Transaction,
    message_id: MessageId,
    inclusion_state: InclusionState,
    updated_transactions: &mut Vec<Transaction>,
    spent_output_ids: &mut Vec<OutputId>,
) {
    transaction.message_id.replace(message_id);
    transaction.inclusion_state = inclusion_state;
    // get spent inputs
    let Essence::Regular(essence) = transaction.payload.essence();
    for input in essence.inputs() {
        if let Input::Utxo(input) = input {
            spent_output_ids.push(*input.output_id());
        }
    }
    updated_transactions.push(transaction);
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::ClientBlockBuilder,
    bee_block::{
        output::{Output, OutputId},
        payload::transaction::TransactionId,
    },
    bee_rest_api::types::responses::OutputResponse,
    Client,
};

use crate::account::{
    handle::AccountHandle,
    operations::syncing::options::SyncOptions,
    types::{address::AddressWithUnspentOutputs, InclusionState, OutputData, Transaction},
    AccountAddress,
};
#[cfg(feature = "events")]
use crate::{
    account::types::OutputDataDto,
    events::types::{NewOutputEvent, SpentOutputEvent, TransactionInclusionEvent, WalletEvent},
    iota_client::bee_block::payload::transaction::dto::TransactionPayloadDto,
};
impl AccountHandle {
    // Set the alias for the account
    pub async fn set_alias(&self, alias: &str) -> crate::Result<()> {
        let mut account = self.write().await;
        account.alias = alias.to_string();
        #[cfg(feature = "storage")]
        self.save(Some(&account)).await?;
        Ok(())
    }

    /// Update account with newly synced data and emit events for outputs
    pub(crate) async fn update_account(
        &self,
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
        unspent_outputs: Vec<OutputData>,
        spent_outputs: Vec<OutputId>,
        spent_output_responses: Vec<OutputResponse>,
        options: &SyncOptions,
    ) -> crate::Result<()> {
        log::debug!("[SYNC] Update account with new synced transactions");

        let network_id = self.client.get_network_id().await?;
        let mut account = self.write().await;
        #[cfg(feature = "events")]
        let account_index = account.index;

        // update used field of the addresses
        for address_with_unspent_outputs in addresses_with_unspent_outputs.iter() {
            if address_with_unspent_outputs.internal {
                let position = account
                    .internal_addresses
                    .binary_search_by_key(
                        &(
                            address_with_unspent_outputs.key_index,
                            address_with_unspent_outputs.internal,
                        ),
                        |a| (a.key_index, a.internal),
                    )
                    .map_err(|_| {
                        crate::Error::AddressNotFoundInAccount(address_with_unspent_outputs.address.to_bech32())
                    })?;
                account.internal_addresses[position].used = true;
            } else {
                let position = account
                    .public_addresses
                    .binary_search_by_key(
                        &(
                            address_with_unspent_outputs.key_index,
                            address_with_unspent_outputs.internal,
                        ),
                        |a| (a.key_index, a.internal),
                    )
                    .map_err(|_| {
                        crate::Error::AddressNotFoundInAccount(address_with_unspent_outputs.address.to_bech32())
                    })?;
                account.public_addresses[position].used = true;
            }
        }

        // Update addresses_with_unspent_outputs
        // get all addresses with balance that we didn't sync because their index is below the address_start_index of
        // the options
        account.addresses_with_unspent_outputs = account
            .addresses_with_unspent_outputs
            .iter()
            .filter(|a| a.key_index < options.address_start_index)
            .cloned()
            .collect();
        // then add all synced addresses with balance, all other addresses that had balance before will then be removed
        // from this list
        account
            .addresses_with_unspent_outputs
            .extend(addresses_with_unspent_outputs);

        // Update spent outputs
        for output_id in spent_outputs {
            if let Some(output) = account.outputs.get(&output_id) {
                // Could also be outputs from other networks after we switched the node, so we check that first
                if output.network_id == network_id {
                    log::debug!("[SYNC] Spent output {}", output_id);
                    account.locked_outputs.remove(&output_id);
                    account.unspent_outputs.remove(&output_id);
                    // Update spent data fields
                    if let Some(output_data) = account.outputs.get_mut(&output_id) {
                        output_data.metadata.is_spent = true;
                        output_data.is_spent = true;
                        #[cfg(feature = "events")]
                        {
                            self.event_emitter.lock().await.emit(
                                account_index,
                                WalletEvent::SpentOutput(SpentOutputEvent {
                                    output: OutputDataDto::from(&*output_data),
                                }),
                            );
                        }
                    }
                }
            }
        }

        // Update output_response if it got spent to include the new metadata
        for output_response in spent_output_responses {
            let transaction_id = TransactionId::from_str(&output_response.metadata.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.metadata.output_index)?;
            if let Some(output_data) = account.outputs.get_mut(&output_id) {
                output_data.metadata = output_response.metadata;
            }
        }

        // Add new synced outputs
        for output_data in unspent_outputs {
            // Insert output, if it's unknown emit the NewOutputEvent
            if account
                .outputs
                .insert(output_data.output_id, output_data.clone())
                .is_none()
            {
                #[cfg(feature = "events")]
                {
                    let transaction = account
                        .incoming_transactions
                        .get(output_data.output_id.transaction_id());
                    self.event_emitter.lock().await.emit(
                        account_index,
                        WalletEvent::NewOutput(NewOutputEvent {
                            output: OutputDataDto::from(&output_data),
                            transaction: transaction
                                .as_ref()
                                .map(|(tx, _inputs)| TransactionPayloadDto::from(tx)),
                            transaction_inputs: transaction.as_ref().map(|(_tx, inputs)| inputs).cloned(),
                        }),
                    );
                }
            };
            if !output_data.is_spent {
                account.unspent_outputs.insert(output_data.output_id, output_data);
            }
        }

        #[cfg(feature = "storage")]
        {
            log::debug!("[SYNC] storing account {} with new synced data", account.alias());
            self.save(Some(&account)).await?;
        }
        Ok(())
    }

    /// Update account with newly synced transactions
    pub(crate) async fn update_account_with_transactions(
        &self,
        updated_transactions: Vec<Transaction>,
        spent_output_ids: Vec<OutputId>,
        output_ids_to_unlock: Vec<OutputId>,
    ) -> crate::Result<()> {
        log::debug!("[SYNC] Update account with new synced transactions");

        let mut account = self.write().await;

        for transaction in updated_transactions {
            match transaction.inclusion_state {
                InclusionState::Confirmed | InclusionState::Conflicting | InclusionState::UnknownPruned => {
                    let transaction_id = transaction.payload.id();
                    account.pending_transactions.remove(&transaction_id);
                    log::debug!(
                        "[SYNC] inclusion_state of {transaction_id} changed to {:?}",
                        transaction.inclusion_state
                    );
                    #[cfg(feature = "events")]
                    {
                        self.event_emitter.lock().await.emit(
                            account.index,
                            WalletEvent::TransactionInclusion(TransactionInclusionEvent {
                                transaction_id,
                                inclusion_state: transaction.inclusion_state,
                            }),
                        );
                    }
                }
                _ => {}
            }
            account
                .transactions
                .insert(transaction.payload.id(), transaction.clone());
        }

        for output_to_unlock in &spent_output_ids {
            if let Some(output) = account.outputs.get_mut(output_to_unlock) {
                output.is_spent = true;
            }
            account.locked_outputs.remove(output_to_unlock);
            account.unspent_outputs.remove(output_to_unlock);
            log::debug!("[SYNC] Unlocked spent output {}", output_to_unlock);
        }

        for output_to_unlock in &output_ids_to_unlock {
            account.locked_outputs.remove(output_to_unlock);
            log::debug!(
                "[SYNC] Unlocked unspent output {} because of a conflicting transaction",
                output_to_unlock
            );
        }

        #[cfg(feature = "storage")]
        {
            log::debug!(
                "[SYNC] storing account {} with new synced transactions",
                account.alias()
            );
            self.save(Some(&account)).await?;
        }
        Ok(())
    }

    /// Update account with newly generated addresses
    pub(crate) async fn update_account_addresses(
        &self,
        internal: bool,
        new_addresses: Vec<AccountAddress>,
    ) -> crate::Result<()> {
        log::debug!("[SYNC] Update account with new synced transactions");

        let mut account = self.write().await;

        // add addresses to the account
        if internal {
            account.internal_addresses.extend(new_addresses);
        } else {
            account.public_addresses.extend(new_addresses);
        };

        #[cfg(feature = "storage")]
        {
            log::debug!("[ADDRESS GENERATION] storing account {}", account.index());
            self.save(Some(&account)).await?;
        }
        Ok(())
    }

    // Should only be called from the AccountManager so all accounts are on the same state
    pub(crate) async fn update_account_with_new_client(&mut self, client: Client) -> crate::Result<()> {
        self.client = client;
        let bech32_hrp = self.client.get_bech32_hrp().await?;
        log::debug!("[UPDATE ACCOUNT WITH NEW CLIENT] new bech32_hrp: {}", bech32_hrp);
        let mut account = self.write().await;
        for address in &mut account.addresses_with_unspent_outputs {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.public_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.internal_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        Ok(())
    }

    /// Update unspent outputs, this function is originally intended for updating recursively synced alias and nft
    /// address outputs
    pub(crate) async fn update_unspent_outputs(&self, output_responses: Vec<OutputResponse>) -> crate::Result<()> {
        let network_id = self.client.get_network_id().await?;
        let mut account = self.write().await;

        for output_response in output_responses.into_iter() {
            let transaction_id = TransactionId::from_str(&output_response.metadata.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.metadata.output_index)?;
            let (_amount, address) = ClientBlockBuilder::get_output_amount_and_address(&output_response.output, None)?;
            // check if we know the transaction that created this output and if we created it (if we store incoming
            // transactions separated, then this check wouldn't be required)
            let remainder = {
                match account.transactions.get(&transaction_id) {
                    Some(tx) => !tx.incoming,
                    None => false,
                }
            };

            let output_data = OutputData {
                output_id,
                output: Output::try_from(&output_response.output)?,
                is_spent: output_response.metadata.is_spent,
                metadata: output_response.metadata,
                address,
                network_id,
                remainder,
                chain: None,
            };
            account.unspent_outputs.entry(output_id).or_insert(output_data);
        }

        Ok(())
    }
}

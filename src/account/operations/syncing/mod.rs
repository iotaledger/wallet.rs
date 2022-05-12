// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod addresses;
pub mod options;
pub(crate) mod outputs;
pub(crate) mod transactions;

use std::{
    str::FromStr,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use iota_client::{
    bee_message::{output::OutputId, payload::transaction::TransactionId},
    bee_rest_api::types::responses::OutputResponse,
};

pub use self::options::SyncOptions;
use crate::account::{
    constants::MIN_SYNC_INTERVAL,
    handle::AccountHandle,
    operations::syncing::transactions::TransactionSyncResult,
    types::{address::AddressWithUnspentOutputs, InclusionState, OutputData},
    AccountBalance,
};
#[cfg(feature = "ledger_nano")]
use crate::secret::SecretManager;

impl AccountHandle {
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        let options = options.unwrap_or_default();
        log::debug!("[SYNC] start syncing with {:?}", options);
        let syc_start_time = Instant::now();

        // prevent syncing the account multiple times simultaneously
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let mut last_synced = self.last_synced.lock().await;
        log::debug!("[SYNC] last time synced before {}ms", time_now - *last_synced);
        if time_now - *last_synced < MIN_SYNC_INTERVAL && !options.force_syncing {
            log::debug!(
                "[SYNC] synced within the latest {} ms, only calculating balance",
                MIN_SYNC_INTERVAL
            );
            // calculate the balance because if we created a transaction in the meantime, the amount for the inputs is
            // not available anymore
            return self.balance().await;
        }

        // sync transactions first so we maybe get confirmed outputs in the syncing process later
        // do we want a field in SyncOptions so it can be skipped?
        let transaction_sync_result = if options.sync_pending_transactions {
            Some(self.sync_pending_transactions().await?)
        } else {
            None
        };

        // one could skip addresses to sync, to sync faster (should we only add a field to the sync option to only sync
        // specific addresses?)
        let addresses_to_sync = self.get_addresses_to_sync(&options).await?;
        log::debug!("[SYNC] addresses_to_sync {}", addresses_to_sync.len());

        // get outputs for addresses and add them also the the addresses_with_unspent_outputs
        let (addresses_with_output_ids, spent_output_ids) =
            self.get_address_output_ids(&options, addresses_to_sync.clone()).await?;

        // get outputs for addresses and add them also the the addresses_with_unspent_outputs
        let (addresses_with_unspent_outputs_and_outputs, output_data) =
            self.get_addresses_outputs(addresses_with_output_ids.clone()).await?;

        // request possible spent outputs
        let (spent_output_responses, _already_known_balance, _loaded_output_responses) =
            self.get_outputs(spent_output_ids.clone(), true).await?;

        let non_ledger_secret_manager = match *self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            // don't automatically consolidate/collect outputs with ledger secret_managers, because they require
            // approval from the user
            SecretManager::LedgerNano(_) | SecretManager::LedgerNanoSimulator(_) => false,
            _ => true,
        };

        // Only consolidates outputs for non ledger accounts, because they require approval from the user
        if options.automatic_output_consolidation && non_ledger_secret_manager {
            self.consolidate_outputs().await?;
        }

        // Only consolidates outputs for non ledger accounts, because they require approval from the user
        if non_ledger_secret_manager {
            self.try_collect_outputs(options.try_collect_outputs).await?;
        }

        // add a field to the sync options to also sync incoming transactions?

        // updates account with balances, output ids, outputs
        self.update_account(
            addresses_with_unspent_outputs_and_outputs,
            output_data,
            transaction_sync_result,
            spent_output_ids,
            spent_output_responses,
            &options,
        )
        .await?;

        let account_balance = self.balance().await?;
        // update last_synced mutex
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        *last_synced = time_now;
        log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
        Ok(account_balance)
    }

    /// Update account with newly synced data
    async fn update_account(
        &self,
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
        new_outputs: Vec<OutputData>,
        transaction_sync_result: Option<TransactionSyncResult>,
        spent_outputs: Vec<OutputId>,
        spent_output_responses: Vec<OutputResponse>,
        options: &SyncOptions,
    ) -> crate::Result<()> {
        log::debug!("[SYNC] Update account with new synced data");

        let network_id = self.client.get_network_id().await?;
        let mut account = self.write().await;
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
                    account.unspent_outputs.remove(&output_id);
                    // Update spent data fields
                    if let Some(output_data) = account.outputs.get_mut(&output_id) {
                        output_data.output_response.is_spent = true;
                        output_data.is_spent = true;
                    }
                }
            }
        }

        // Update output_response if it got spent to include the new metadata
        for output_response in spent_output_responses {
            let transaction_id = TransactionId::from_str(&output_response.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.output_index)?;
            if let Some(output_data) = account.outputs.get_mut(&output_id) {
                output_data.output_response = output_response;
            }
        }

        // Add new synced outputs
        for output in new_outputs {
            account.outputs.insert(output.output_id, output.clone());
            if !output.is_spent {
                account.unspent_outputs.insert(output.output_id, output);
            }
        }

        // Update data from synced transactions
        if let Some(transaction_sync_result) = transaction_sync_result {
            for transaction in transaction_sync_result.updated_transactions {
                match transaction.inclusion_state {
                    InclusionState::Confirmed | InclusionState::Conflicting => {
                        account.pending_transactions.remove(&transaction.payload.id());
                    }
                    _ => {}
                }
                account.transactions.insert(transaction.payload.id(), transaction);
            }

            for output_to_unlock in transaction_sync_result.spent_output_ids {
                if let Some(output) = account.outputs.get_mut(&output_to_unlock) {
                    output.is_spent = true;
                }
                account.locked_outputs.remove(&output_to_unlock);
                account.unspent_outputs.remove(&output_to_unlock);
                log::debug!("[SYNC] Unlocked spent output {}", output_to_unlock);
            }
            for output_to_unlock in transaction_sync_result.output_ids_to_unlock {
                if let Some(output) = account.outputs.get_mut(&output_to_unlock) {
                    output.is_spent = true;
                }
                account.locked_outputs.remove(&output_to_unlock);
                log::debug!(
                    "[SYNC] Unlocked unspent output {} because of a conflicting transaction",
                    output_to_unlock
                );
            }
        }
        #[cfg(feature = "storage")]
        {
            log::debug!("[SYNC] storing account {} with new synced data", account.alias());
            self.save(Some(&account)).await?;
        }
        // println!("{:#?}", account);
        Ok(())
    }
}

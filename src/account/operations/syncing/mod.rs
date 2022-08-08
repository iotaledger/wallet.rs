// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod addresses;
pub mod options;
pub(crate) mod outputs;
pub(crate) mod transactions;

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use iota_client::{
    api_types::responses::OutputResponse,
    block::{Block, BlockId},
};

pub use self::options::SyncOptions;
use crate::account::{constants::MIN_SYNC_INTERVAL, handle::AccountHandle, AccountBalance};

impl AccountHandle {
    /// Retries (promotes or reattaches) a block for provided block id until it's included (referenced by a
    /// milestone). This function is re-exported from the client library and default interval is as defined in iota.rs.
    /// Returns the included block at first position and additional reattached blocks
    pub async fn retry_until_included(
        &self,
        block_id: &BlockId,
        interval: Option<u64>,
        max_attempts: Option<u64>,
    ) -> crate::Result<Vec<(BlockId, Block)>> {
        Ok(self
            .client
            .retry_until_included(block_id, interval, max_attempts)
            .await?)
    }

    /// Sync the account by fetching new information from the nodes. Will also retry pending transactions
    /// if necessary.
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

        // one could skip addresses to sync, to sync faster (should we only add a field to the sync option to only sync
        // specific addresses?)
        let addresses_to_sync = self.get_addresses_to_sync(&options).await?;
        log::debug!("[SYNC] addresses_to_sync {}", addresses_to_sync.len());

        // get outputs for addresses and add them also the the addresses_with_unspent_outputs
        let (addresses_with_output_ids, spent_or_not_synced_output_ids) =
            self.get_address_output_ids(&options, addresses_to_sync.clone()).await?;

        // get outputs for addresses and add them also the the addresses_with_unspent_outputs
        let (addresses_with_unspent_outputs_and_outputs, output_data) =
            self.get_addresses_outputs(addresses_with_output_ids.clone()).await?;

        // request possible spent outputs
        let (spent_or_not_synced_output_responses, _loaded_output_responses) =
            self.get_outputs(spent_or_not_synced_output_ids.clone(), true).await?;

        if options.sync_incoming_transactions {
            let transaction_ids = output_data
                .iter()
                .map(|output| *output.output_id.transaction_id())
                .collect();
            // Request and store transaction payload for newly received unspent outputs
            self.request_incoming_transaction_data(transaction_ids).await?;
        }

        // updates account with balances, output ids, outputs
        self.update_account(
            addresses_with_unspent_outputs_and_outputs,
            output_data,
            spent_or_not_synced_output_ids,
            spent_or_not_synced_output_responses,
            &options,
        )
        .await?;

        // Sync transactions after updating account with outputs, so we can use them to check the transaction status
        if options.sync_pending_transactions {
            self.sync_pending_transactions().await?;
        };

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
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use crate::{
    account::handle::AccountHandle,
    account_manager::{AccountManager, SyncOptions},
};

impl AccountManager {
    /// Find accounts with unspent outputs
    /// `account_gap_limit` defines how many accounts without unspent outputs will be
    /// checked, if an account has unspent outputs, the counter is reset
    /// `address_gap_limit` defines how many addresses without unspent outputs will be checked in each account, if an
    /// address has unspent outputs, the counter is reset
    /// address_start_index and force_syncing will be overwritten in sync_options to not skip addresses, but also don't
    /// send duplicated requests
    pub async fn recover_accounts(
        &self,
        account_gap_limit: u32,
        address_gap_limit: u32,
        sync_options: Option<SyncOptions>,
    ) -> crate::Result<Vec<AccountHandle>> {
        log::debug!("[recover_accounts]");
        let start_time = Instant::now();
        let mut max_account_index_to_keep = None;

        // Search for addresses in current accounts
        for account_handle in self.accounts.read().await.iter() {
            // If the gap limit is 0, there is no need to search for funds
            if address_gap_limit > 0 {
                account_handle
                    .search_addresses_with_outputs(address_gap_limit, sync_options.clone())
                    .await?;
            }
            let account_index = *account_handle.read().await.index();
            match max_account_index_to_keep {
                Some(max_account_index) => {
                    if account_index > max_account_index {
                        max_account_index_to_keep = Some(account_index);
                    }
                }
                None => max_account_index_to_keep = Some(account_index),
            }
        }

        // Don't return possible errors here already, because we would then still have empty accounts
        let new_accounts_discovery_result = self
            .search_new_accounts(
                account_gap_limit,
                address_gap_limit,
                &mut max_account_index_to_keep,
                sync_options.clone(),
            )
            .await;

        // remove accounts without outputs
        let mut accounts = self.accounts.write().await;
        let mut new_accounts = Vec::new();
        for account_handle in accounts.iter() {
            let account_index = *account_handle.read().await.index();
            let mut keep_account = false;
            if let Some(max_account_index_to_keep) = max_account_index_to_keep {
                if account_index <= max_account_index_to_keep {
                    new_accounts.push((account_index, account_handle.clone()));
                    keep_account = true;
                }
            }

            if !keep_account {
                // accounts are stored during syncing, delete the empty accounts again
                #[cfg(feature = "storage")]
                {
                    log::debug!("[recover_accounts] delete emtpy account {}", account_index);
                    self.storage_manager.lock().await.remove_account(account_index).await?;
                }
            }
        }
        new_accounts.sort_by_key(|(index, _acc)| *index);
        *accounts = new_accounts.into_iter().map(|(_, acc)| acc).collect();
        drop(accounts);

        // Handle result after cleaning up the empty accounts
        new_accounts_discovery_result?;

        log::debug!("[recover_accounts] finished in {:?}", start_time.elapsed());
        Ok(self.accounts.read().await.clone())
    }

    /// Generate new accounts and search for unspent outputs
    async fn search_new_accounts(
        &self,
        account_gap_limit: u32,
        address_gap_limit: u32,
        max_account_index_to_keep: &mut Option<u32>,
        sync_options: Option<SyncOptions>,
    ) -> crate::Result<()> {
        loop {
            log::debug!("[recover_accounts] generating {account_gap_limit} new accounts");

            // Generate account with addresses and get their outputs in parallel
            let mut tasks = Vec::new();
            for _ in 0..account_gap_limit {
                let mut new_account = self.create_account();
                let sync_options_ = sync_options.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let new_account = new_account.finish().await?;
                        let account_outputs_count = new_account
                            .search_addresses_with_outputs(address_gap_limit, sync_options_.clone())
                            .await?;
                        let account_index = *new_account.read().await.index();
                        Ok((account_index, account_outputs_count))
                    })
                    .await
                });
            }

            let results: Vec<crate::Result<(u32, usize)>> = futures::future::try_join_all(tasks).await?;
            let mut total_account_outputs_count = 0;
            for res in results {
                let (account_index, outputs_count): (u32, usize) = res?;
                total_account_outputs_count += outputs_count;

                if outputs_count != 0 {
                    match *max_account_index_to_keep {
                        Some(max_account_index) => {
                            if account_index > max_account_index {
                                *max_account_index_to_keep = Some(account_index);
                            }
                        }
                        None => *max_account_index_to_keep = Some(account_index),
                    }
                }
            }

            // If all accounts in this round have no outputs, we break
            if total_account_outputs_count == 0 {
                break;
            }
        }

        Ok(())
    }
}

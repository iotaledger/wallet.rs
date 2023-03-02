// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use instant::Instant;

use crate::{
    account::handle::AccountHandle,
    account_manager::{AccountManager, SyncOptions},
    task,
};

impl AccountManager {
    /// Find accounts with unspent outputs.
    ///
    /// Arguments:
    ///
    /// * `account_start_index`: The index of the first account to search for.
    /// * `account_gap_limit`: The number of accounts to search for, after the last account with unspent outputs.
    /// * `address_gap_limit`: The number of addresses to search for, after the last address with unspent outputs, in
    ///   each account.
    /// * `sync_options`: Optional parameter to specify the sync options. The `address_start_index` and `force_syncing`
    ///   fields will be overwritten to skip existing addresses.
    ///
    /// Returns:
    ///
    /// A vector of AccountHandle
    pub async fn recover_accounts(
        &self,
        account_start_index: u32,
        account_gap_limit: u32,
        address_gap_limit: u32,
        sync_options: Option<SyncOptions>,
    ) -> crate::Result<Vec<AccountHandle>> {
        log::debug!("[recover_accounts]");
        let start_time = Instant::now();
        let mut max_account_index_to_keep = None;

        // Search for addresses in current accounts
        #[allow(clippy::significant_drop_in_scrutinee)]
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

        // Create accounts below account_start_index, because we don't want to have gaps in the accounts, but we also
        // don't want to sync them
        for _ in max_account_index_to_keep.unwrap_or(0)..account_start_index {
            // Don't return possible errors here, because we could then still have empty accounts
            let _ = self.create_account().finish().await;
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
        let mut new_accounts = Vec::new();
        let mut accounts = self.accounts.write().await;

        #[allow(clippy::significant_drop_in_scrutinee)]
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
                    log::debug!("[recover_accounts] delete empty account {}", account_index);
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
        let mut updated_account_gap_limit = account_gap_limit;
        loop {
            log::debug!("[recover_accounts] generating {updated_account_gap_limit} new accounts");

            // Generate account with addresses and get their outputs in parallel
            let mut tasks = Vec::new();
            for _ in 0..updated_account_gap_limit {
                let mut new_account = self.create_account();
                let sync_options_ = sync_options.clone();
                tasks.push(async move {
                    task::spawn(async move {
                        let new_account = new_account.finish().await?;
                        let account_outputs_count = new_account
                            .search_addresses_with_outputs(address_gap_limit, sync_options_)
                            .await?;
                        let account_index = *new_account.read().await.index();
                        Ok((account_index, account_outputs_count))
                    })
                    .await
                });
            }

            let results: Vec<crate::Result<(u32, usize)>> = futures::future::try_join_all(tasks).await?;

            let mut new_accounts_with_outputs = 0;
            let mut highest_account_index = 0;
            for res in results {
                let (account_index, outputs_count): (u32, usize) = res?;
                if outputs_count != 0 {
                    new_accounts_with_outputs += 1;

                    match *max_account_index_to_keep {
                        Some(max_account_index) => {
                            if account_index > max_account_index {
                                *max_account_index_to_keep = Some(account_index);
                            }
                        }
                        None => *max_account_index_to_keep = Some(account_index),
                    }
                }

                if account_index > highest_account_index {
                    highest_account_index = account_index;
                }
            }

            // Break if there is no new account with outputs
            if new_accounts_with_outputs == 0 {
                break;
            }

            // Update account_gap_limit to only create so many new accounts, that we would check the initial provided
            // account_gap_limit amount of empty accounts
            if let Some(max_account_index_to_keep) = &max_account_index_to_keep {
                let empty_accounts_in_row = highest_account_index - max_account_index_to_keep;
                log::debug!("[recover_accounts] empty_accounts_in_row {empty_accounts_in_row}");
                updated_account_gap_limit = account_gap_limit - empty_accounts_in_row;
            }
        }

        Ok(())
    }
}

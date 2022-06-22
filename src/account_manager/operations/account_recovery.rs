// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use crate::{
    account::handle::AccountHandle,
    account_manager::{AccountBalance, AccountManager},
};

impl AccountManager {
    /// Find accounts with unspent outputs
    /// `account_gap_limit` defines how many accounts without unspent outputs will be
    /// checked, if an account has unspent outputs, the counter is reset
    /// `address_gap_limit` defines how many addresses without unspent outputs will be checked in each account, if an
    /// address has unspent outputs, the counter is reset
    pub async fn recover_accounts(
        &self,
        account_gap_limit: u32,
        address_gap_limit: u32,
    ) -> crate::Result<Vec<AccountHandle>> {
        log::debug!("[recover_accounts]");
        let start_time = Instant::now();
        let mut max_account_index_to_keep = None;

        // Search for addresses in current accounts
        for account_handle in self.accounts.read().await.iter() {
            // If the gap limit is 0, there is no need to search for funds
            if address_gap_limit > 0 {
                account_handle.search_addresses_with_funds(address_gap_limit).await?;
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

        loop {
            log::debug!("[recover_accounts] generating {account_gap_limit} new accounts");

            // Generate account with addresses and get their balance in parallel
            let mut tasks = Vec::new();
            for _ in 0..account_gap_limit {
                let mut new_account = self.create_account();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let new_account = new_account.finish().await?;
                        let account_balance = new_account.search_addresses_with_funds(address_gap_limit).await?;
                        let account_index = *new_account.read().await.index();
                        Ok((account_index, account_balance))
                    })
                    .await
                });
            }

            let results: Vec<crate::Result<(u32, AccountBalance)>> = futures::future::try_join_all(tasks).await?;
            let mut total_account_balances = 0;
            for res in results {
                let (account_index, account_balance): (u32, AccountBalance) = res?;
                total_account_balances += account_balance.total;

                if account_balance.total != 0 {
                    match max_account_index_to_keep {
                        Some(max_account_index) => {
                            if account_index > max_account_index {
                                max_account_index_to_keep = Some(account_index);
                            }
                        }
                        None => max_account_index_to_keep = Some(account_index),
                    }
                }
            }

            // If all accounts in this round have no balance, we break
            if total_account_balances == 0 {
                break;
            }
        }

        // remove accounts without balance
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

        log::debug!("[recover_accounts] finished in {:?}", start_time.elapsed());
        Ok(self.accounts.read().await.clone())
    }
}

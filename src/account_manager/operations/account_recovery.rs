// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::handle::AccountHandle, account_manager::AccountManager};

use std::collections::HashSet;

/// Find accounts with balances
/// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
/// has balance, the counter is reset
/// `account_gap_limit` defines how many accounts without balance will be
/// checked, if an account has balance, the counter is reset
pub async fn recover_accounts(
    account_manager: &AccountManager,
    address_gap_limit: usize,
    account_gap_limit: usize,
) -> crate::Result<Vec<AccountHandle>> {
    log::debug!("[recover_accounts]");
    let mut account_indexes_to_keep = HashSet::new();

    // Search for addresses in current accounts
    for account_handle in account_manager.accounts.read().await.iter() {
        account_handle.search_addresses_with_funds(address_gap_limit).await?;
        let account_index = *account_handle.read().await.index();
        account_indexes_to_keep.insert(account_index);
    }

    // Count accounts with zero balances in a row
    let mut zero_balance_accounts_in_row = 0;
    let mut generated_accounts = Vec::new();
    loop {
        log::debug!("[recover_accounts] generating new account");
        let new_account = account_manager.create_account().finish().await?;
        let account_balance = new_account.search_addresses_with_funds(address_gap_limit).await?;
        generated_accounts.push((new_account, account_balance.clone()));
        if account_balance.total == 0 {
            zero_balance_accounts_in_row += 1;
            if zero_balance_accounts_in_row >= account_gap_limit {
                break;
            }
        } else {
            // reset if we found an account with balance
            zero_balance_accounts_in_row = 0;
        }
    }
    // iterate reversed to ignore all latest accounts that have no balance, but add all accounts that are below one
    // with balance
    let mut got_account_with_balance = false;
    for (account_handle, account_balance) in generated_accounts.iter().rev() {
        let account = account_handle.read().await;
        if got_account_with_balance || account_balance.total != 0 {
            got_account_with_balance = true;
            account_indexes_to_keep.insert(*account_handle.read().await.index());
        }
    }

    // remove accounts without balance
    let mut accounts = account_manager.accounts.write().await;
    let mut new_accounts = Vec::new();
    for account_handle in accounts.iter() {
        let account_index = *account_handle.read().await.index();
        if account_indexes_to_keep.contains(&account_index) {
            new_accounts.push((account_index, account_handle.clone()));
        } else {
            // accounts are stored during syncing, delete the empty accounts again
            #[cfg(feature = "storage")]
            log::debug!("[recover_accounts] delete emtpy account {}", account_index);
            crate::storage::manager::get()
                .await?
                .lock()
                .await
                .remove_account(account_index)
                .await?;
        }
    }
    new_accounts.sort_by_key(|(index, acc)| *index);
    *accounts = new_accounts.into_iter().map(|(_, acc)| acc).collect();
    drop(accounts);

    Ok(account_manager.accounts.read().await.clone())
}

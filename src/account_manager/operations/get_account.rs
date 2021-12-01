// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, types::AccountIdentifier},
    account_manager::AccountManager,
};

/// Get an account with an AccountIdentifier
pub async fn get_account<I: Into<AccountIdentifier>>(
    account_manager: &AccountManager,
    identifier: I,
) -> crate::Result<AccountHandle> {
    log::debug!("get_account");
    let account_id = identifier.into();
    let accounts = account_manager.accounts.read().await;

    match account_id {
        AccountIdentifier::Id(id) => {
            for account_handle in accounts.iter() {
                let account = account_handle.read().await;
                if account.id() == &id {
                    return Ok(account_handle.clone());
                }
            }
        }
        AccountIdentifier::Index(index) => {
            for account_handle in accounts.iter() {
                let account = account_handle.read().await;
                if account.index() == &index {
                    return Ok(account_handle.clone());
                }
            }
        }
        AccountIdentifier::Alias(alias) => {
            for account_handle in accounts.iter() {
                let account = account_handle.read().await;
                if account.alias() == &alias {
                    return Ok(account_handle.clone());
                }
            }
        }
    };
    Err(crate::Error::AccountNotFound)
}

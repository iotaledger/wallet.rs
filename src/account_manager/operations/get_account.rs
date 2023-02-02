// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, types::AccountIdentifier},
    account_manager::AccountManager,
};

impl AccountManager {
    /// Get an account with an AccountIdentifier
    pub async fn get_account<I: Into<AccountIdentifier>>(&self, identifier: I) -> crate::Result<AccountHandle> {
        let account_id = identifier.into();
        let accounts = self.accounts.read().await;

        match &account_id {
            AccountIdentifier::Index(index) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.index() == index {
                        return Ok(account_handle.clone());
                    }
                }
            }
            AccountIdentifier::Alias(alias) => {
                for account_handle in accounts.iter() {
                    let account = account_handle.read().await;
                    if account.alias() == alias {
                        return Ok(account_handle.clone());
                    }
                }
            }
        };
        Err(crate::Error::AccountNotFound(serde_json::to_string(&account_id)?))
    }
}

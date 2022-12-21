// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_manager::AccountManager,
    client::secret::{LedgerNanoStatus, SecretManager},
};

impl AccountManager {
    /// Get the ledger nano status
    pub async fn get_ledger_nano_status(&self) -> crate::Result<LedgerNanoStatus> {
        if let SecretManager::LedgerNano(ledger) = &*self.secret_manager.read().await {
            Ok(ledger.get_ledger_nano_status().await)
        } else {
            Err(crate::client::Error::SecretManagerMismatch.into())
        }
    }
}

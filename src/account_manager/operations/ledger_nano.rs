// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::secret::{LedgerStatus, SecretManager};

use crate::account_manager::AccountManager;

impl AccountManager {
    /// Get the ledger nano status
    pub async fn get_ledger_status(&self) -> crate::Result<LedgerStatus> {
        if let SecretManager::LedgerNano(ledger) = &*self.secret_manager.read().await {
            Ok(ledger.get_ledger_status().await)
        } else {
            Err(iota_client::Error::SecretManagerMismatch.into())
        }
    }
}

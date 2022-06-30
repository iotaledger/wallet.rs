// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::secret::{LedgerStatus, SecretManager};

use crate::account_manager::AccountManager;

impl AccountManager {
    /// Get the ledger nano status
    pub async fn get_ledger_status(&self) -> crate::Result<LedgerStatus> {
        match &*self.secret_manager.read().await {
            SecretManager::LedgerNano(ledger) | SecretManager::LedgerNanoSimulator(ledger) => {
                Ok(ledger.get_ledger_status().await)
            }
            _ => Err(iota_client::Error::SecretManagerMismatch.into()),
        }
    }
}

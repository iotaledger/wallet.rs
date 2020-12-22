// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use crate::account::AccountIdentifier;

use std::path::{Path, PathBuf};

/// Stronghold storage adapter.
pub struct StrongholdStorageAdapter {
    path: PathBuf,
}

impl StrongholdStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for StrongholdStorageAdapter {
    async fn get(&self, account_id: &AccountIdentifier) -> crate::Result<String> {
        let account = crate::stronghold::get_account(&self.path, account_id).await?;
        Ok(account)
    }

    async fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let accounts = crate::stronghold::get_accounts(&self.path).await?;
        Ok(accounts)
    }

    async fn set(&self, account_id: &AccountIdentifier, account: String) -> crate::Result<()> {
        crate::stronghold::store_account(&self.path, account_id, account).await?;
        Ok(())
    }

    async fn remove(&self, account_id: &AccountIdentifier) -> crate::Result<()> {
        crate::stronghold::remove_account(&self.path, account_id).await?;
        Ok(())
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use crate::account::AccountIdentifier;

use std::path::{Path, PathBuf};

/// The storage id.
pub const STORAGE_ID: &str = "STRONGHOLD";

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

fn storage_err<E: ToString>(error: E) -> crate::Error {
    crate::Error::Storage(error.to_string())
}

#[async_trait::async_trait]
impl StorageAdapter for StrongholdStorageAdapter {
    fn id(&self) -> &'static str {
        STORAGE_ID
    }

    async fn get(&mut self, account_id: &AccountIdentifier) -> crate::Result<String> {
        let account = crate::stronghold::get_account(&self.path, account_id)
            .await
            .map_err(storage_err)?;
        Ok(account)
    }

    async fn get_all(&mut self) -> crate::Result<std::vec::Vec<String>> {
        let accounts = crate::stronghold::get_accounts(&self.path).await.map_err(storage_err)?;
        Ok(accounts)
    }

    async fn set(&mut self, account_id: &AccountIdentifier, account: String) -> crate::Result<()> {
        crate::stronghold::store_account(&self.path, account_id, account)
            .await
            .map_err(storage_err)?;
        Ok(())
    }

    async fn remove(&mut self, account_id: &AccountIdentifier) -> crate::Result<()> {
        crate::stronghold::remove_account(&self.path, account_id)
            .await
            .map_err(storage_err)?;
        Ok(())
    }
}

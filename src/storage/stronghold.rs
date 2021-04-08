// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

fn storage_err(error: crate::stronghold::Error) -> crate::Error {
    match error {
        crate::stronghold::Error::RecordNotFound => crate::Error::RecordNotFound,
        _ => crate::Error::Storage(error.to_string()),
    }
}

#[async_trait::async_trait]
impl StorageAdapter for StrongholdStorageAdapter {
    fn id(&self) -> &'static str {
        STORAGE_ID
    }

    async fn get(&self, key: &str) -> crate::Result<String> {
        crate::stronghold::get_record(&self.path, key)
            .await
            .map_err(storage_err)
    }

    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        crate::stronghold::store_record(&self.path, key, record)
            .await
            .map_err(storage_err)?;
        Ok(())
    }

    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        for (key, value) in records {
            self.set(&key, value).await?;
        }
        Ok(())
    }

    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        crate::stronghold::remove_record(&self.path, key)
            .await
            .map_err(storage_err)?;
        Ok(())
    }
}

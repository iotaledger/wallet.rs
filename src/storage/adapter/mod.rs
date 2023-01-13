// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// RocksDB storage adapter.
#[cfg(feature = "rocksdb")]
pub mod rocksdb;

use std::collections::HashMap;

fn storage_err<E: ToString>(error: E) -> crate::Error {
    crate::Error::Storage(error.to_string())
}

/// The storage adapter.
#[async_trait::async_trait]
pub trait StorageAdapter: std::fmt::Debug {
    /// Gets the storage identifier (used internally on the default storage adapters)
    fn id(&self) -> &'static str {
        "custom-adapter"
    }
    /// Gets the record associated with the given key from the storage.
    async fn get(&self, key: &str) -> crate::Result<String>;
    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()>;
    /// Batch write.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()>;
    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()>;
}

#[derive(Debug, Default)]
/// A storage adapter that stores data in memory.
pub struct Memory(HashMap<String, String>);

#[async_trait::async_trait]
impl StorageAdapter for Memory {
    async fn get(&self, key: &str) -> crate::Result<String> {
        self.0.get(key).ok_or(storage_err(key)).cloned()
    }

    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        self.0.insert(key.to_string(), record);
        Ok(())
    }

    /// Batch write.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        self.0.extend(records.into_iter());
        Ok(())
    }

    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.0.remove(key);
        Ok(())
    }
}

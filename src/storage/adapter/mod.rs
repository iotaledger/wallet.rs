// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// RocksDB storage adapter.
pub mod rocksdb;

use std::collections::HashMap;

/// The storage adapter.
#[async_trait::async_trait]
pub trait StorageAdapter {
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

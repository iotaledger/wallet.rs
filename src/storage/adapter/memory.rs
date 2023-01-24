// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::{storage_err, StorageAdapter};

/// A storage adapter that stores data in memory.
#[derive(Debug, Default)]
pub struct Memory(HashMap<String, String>);

#[async_trait::async_trait]
impl StorageAdapter for Memory {
    async fn get(&self, key: &str) -> crate::Result<String> {
        self.0.get(key).ok_or_else(|| storage_err(key)).cloned()
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

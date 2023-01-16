// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;

use gloo_storage::LocalStorage;

/// The storage id.
pub const STORAGE_ID: &str = "Wasm";

/// Wasm storage adapter using the browser local storage
#[derive(Debug)]
pub struct WasmAdapter(LocalStorage);

impl WasmAdapter {
    /// Initialises the storage adapter.
    pub fn new() -> crate::Result<Self> {
        Ok(Self(LocalStorage::new()))
    }
}

#[async_trait::async_trait]
impl StorageAdapter for WasmAdapter {
    async fn get(&self, key: &str) -> crate::Result<String> {
        self.0.get(key)
    }

    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        self.0.set(key, record)
    }

    /// Batch write.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        records.into_iter().map(|s| self.set(s.0, s.1));
        Ok(())
    }

    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.0.delete(key)
    }
}

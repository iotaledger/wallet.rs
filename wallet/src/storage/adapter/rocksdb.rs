// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::Path, sync::Arc};

use rocksdb::{DBCompressionType, Options, WriteBatch, DB};
use tokio::sync::Mutex;

use super::StorageAdapter;

/// The storage id.
pub const STORAGE_ID: &str = "RocksDB";

// fn storage_err<E: ToString>(error: E) -> crate::Error {
//     crate::Error::Storage(error.to_string())
// }

/// Key value storage adapter.
#[derive(Debug)]
pub struct RocksdbStorageAdapter {
    db: Arc<Mutex<DB>>,
}

impl RocksdbStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new(path: impl AsRef<Path>) -> crate::Result<Self> {
        let mut opts = Options::default();
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open(&opts, path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for RocksdbStorageAdapter {
    fn id(&self) -> &'static str {
        STORAGE_ID
    }

    /// Gets the record associated with the given key from the storage.
    async fn get(&self, key: &str) -> crate::Result<Option<String>> {
        Ok(self
            .db
            .lock()
            .await
            .get(key.as_bytes())?
            .map(|r| String::from_utf8_lossy(&r).to_string()))
    }

    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        self.db.lock().await.put(key.as_bytes(), record.as_bytes())?;
        Ok(())
    }

    /// Batch writes records to the storage.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        let mut batch = WriteBatch::default();
        for (key, value) in records {
            batch.put(key.as_bytes(), value.as_bytes());
        }
        self.db.lock().await.write(batch)?;
        Ok(())
    }

    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.db.lock().await.delete(key.as_bytes())?;
        Ok(())
    }
}

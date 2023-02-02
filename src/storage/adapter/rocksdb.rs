// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::Path, sync::Arc};

use rocksdb::{DBCompressionType, Options, WriteBatch, DB};
use tokio::sync::Mutex;

use super::{storage_err, StorageAdapter};

/// The storage id.
pub const STORAGE_ID: &str = "RocksDB";

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
        let db = DB::open(&opts, path).map_err(storage_err)?;
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
    async fn get(&self, key: &str) -> crate::Result<String> {
        match self.db.lock().await.get(key.as_bytes()) {
            Ok(Some(r)) => Ok(String::from_utf8_lossy(&r).to_string()),
            Ok(None) => Err(crate::Error::RecordNotFound(key.to_string())),
            Err(e) => Err(storage_err(e)),
        }
    }

    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        self.db
            .lock()
            .await
            .put(key.as_bytes(), record.as_bytes())
            .map_err(storage_err)?;
        Ok(())
    }

    /// Batch writes records to the storage.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        let mut batch = WriteBatch::default();
        for (key, value) in records {
            batch.put(key.as_bytes(), value.as_bytes());
        }
        self.db.lock().await.write(batch).map_err(storage_err)?;
        Ok(())
    }

    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.db.lock().await.delete(key.as_bytes()).map_err(storage_err)?;
        Ok(())
    }
}

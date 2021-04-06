// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use rocksdb::{DBCompressionType, Options, DB};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// The storage id.
pub const STORAGE_ID: &str = "RocksDB";

/// Key value storage adapter.
pub struct RocksdbStorageAdapter {
    db: Arc<Mutex<DB>>,
}

fn storage_err<E: ToString>(error: E) -> crate::Error {
    crate::Error::Storage(error.to_string())
}

impl RocksdbStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new(path: impl AsRef<Path>) -> crate::Result<Self> {
        let mut opts = Options::default();
        opts.set_compression_type(DBCompressionType::Lz4);
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

    async fn get(&self, key: &str) -> crate::Result<String> {
        match self.db.lock().await.get(key.as_bytes()) {
            Ok(Some(r)) => Ok(String::from_utf8_lossy(&r).to_string()),
            Ok(None) => Err(crate::Error::RecordNotFound),
            Err(e) => Err(storage_err(e)),
        }
    }

    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        self.db
            .lock()
            .await
            .put(key.as_bytes(), record.as_bytes())
            .map_err(storage_err)?;
        Ok(())
    }

    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.db.lock().await.delete(key.as_bytes()).map_err(storage_err)?;
        Ok(())
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use chrono::prelude::*;
use rusqlite::{
    params,
    types::{ToSqlOutput, Value},
    Connection, NO_PARAMS,
};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// The storage id.
pub const STORAGE_ID: &str = "SQLITE";

/// Key value storage adapter.
pub struct SqliteStorageAdapter {
    connection: Arc<Mutex<Connection>>,
}

fn storage_err<E: ToString>(error: E) -> crate::Error {
    crate::Error::Storage(error.to_string())
}

impl SqliteStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new(path: impl AsRef<Path>) -> crate::Result<Self> {
        let connection = Connection::open(path.as_ref()).map_err(storage_err)?;

        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS iota_wallet_records (
                    key TEXT NOT NULL UNIQUE,
                    value TEXT,
                    created_at INTEGER
                )",
                NO_PARAMS,
            )
            .map_err(storage_err)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for SqliteStorageAdapter {
    fn id(&self) -> &'static str {
        STORAGE_ID
    }

    async fn get(&self, key: &str) -> crate::Result<String> {
        let sql = "SELECT value FROM iota_wallet_records WHERE key = ?1 LIMIT 1";
        let params = vec![ToSqlOutput::Owned(Value::Text(key.to_string()))];

        let connection = self.connection.lock().await;
        let mut query = connection.prepare(&sql).map_err(storage_err)?;
        let results = query
            .query_and_then(params, |row| row.get(0))
            .map_err(storage_err)?
            .collect::<Vec<rusqlite::Result<String>>>();
        results
            .first()
            .map(|val| val.as_ref().unwrap().to_string())
            .ok_or(crate::Error::RecordNotFound)
    }

    async fn set(&mut self, key: &str, record: String) -> crate::Result<()> {
        let connection = self.connection.lock().await;
        connection
            .execute(
                "INSERT OR REPLACE INTO iota_wallet_records VALUES (?1, ?2, ?3)",
                params![key, record, Local::now().timestamp()],
            )
            .map_err(|_| crate::Error::Storage("failed to insert data".into()))?;
        Ok(())
    }

    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        let sql = "DELETE FROM iota_wallet_records WHERE key = ?1";
        let params = vec![ToSqlOutput::Owned(Value::Text(key.to_string()))];

        let connection = self.connection.lock().await;
        connection
            .execute(&sql, params)
            .map_err(|_| crate::Error::Storage("failed to delete data".into()))?;
        Ok(())
    }
}

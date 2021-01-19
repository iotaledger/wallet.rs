// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::StorageAdapter;
use crate::account::AccountIdentifier;
use chrono::prelude::*;
use rusqlite::{
    params,
    types::{ToSqlOutput, Value},
    Connection, NO_PARAMS,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

/// The storage id.
pub const STORAGE_ID: &str = "SQLITE";

/// Key value storage adapter.
pub struct SqliteStorageAdapter {
    table_name: String,
    connection: Arc<Mutex<Connection>>,
}

fn storage_err<E: ToString>(error: E) -> crate::Error {
    crate::Error::Storage(error.to_string())
}

impl SqliteStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new(path: impl AsRef<Path>, table_name: impl AsRef<str>) -> crate::Result<Self> {
        let connection = Connection::open(path.as_ref()).map_err(storage_err)?;

        connection
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    key TEXT NOT NULL UNIQUE,
                    value TEXT,
                    created_at INTEGER
                )",
                    table_name.as_ref()
                ),
                NO_PARAMS,
            )
            .map_err(storage_err)?;

        Ok(Self {
            table_name: table_name.as_ref().to_string(),
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for SqliteStorageAdapter {
    fn id(&self) -> &'static str {
        STORAGE_ID
    }

    async fn get(&mut self, account_id: &AccountIdentifier) -> crate::Result<String> {
        let (sql, params) = match account_id {
            AccountIdentifier::Id(id) => (
                format!("SELECT value FROM {} WHERE key = ?1 LIMIT 1", self.table_name),
                vec![ToSqlOutput::Owned(Value::Text(id.clone()))],
            ),
            AccountIdentifier::Index(index) => (
                format!("SELECT value FROM {} LIMIT 1 OFFSET {}", self.table_name, index),
                vec![],
            ),
        };

        let connection = self.connection.lock().expect("failed to get connection lock");
        let mut query = connection.prepare(&sql).map_err(storage_err)?;
        let results = query
            .query_and_then(params, |row| row.get(0))
            .map_err(storage_err)?
            .collect::<Vec<rusqlite::Result<String>>>();
        let account = results
            .first()
            .map(|val| val.as_ref().unwrap().to_string())
            .ok_or(crate::Error::AccountNotFound)?;
        Ok(account)
    }

    async fn get_all(&mut self) -> crate::Result<std::vec::Vec<String>> {
        let connection = self.connection.lock().expect("failed to get connection lock");
        let mut query = connection
            .prepare(&format!("SELECT value FROM {} ORDER BY created_at", self.table_name))
            .map_err(storage_err)?;
        let accounts = query
            .query_and_then(NO_PARAMS, |row| row.get(0))
            .map_err(storage_err)?
            .map(|val| val.unwrap())
            .collect::<Vec<String>>();
        Ok(accounts)
    }

    async fn set(&mut self, account_id: &AccountIdentifier, account: String) -> crate::Result<()> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(crate::Error::Storage("only Id is supported".into())),
        };
        let connection = self.connection.lock().expect("failed to get connection lock");
        connection
            .execute(
                &format!("INSERT OR REPLACE INTO {} VALUES (?1, ?2, ?3)", self.table_name),
                params![id, account, Local::now().timestamp()],
            )
            .map_err(|_| crate::Error::Storage("failed to insert data".into()))?;
        Ok(())
    }

    async fn remove(&mut self, account_id: &AccountIdentifier) -> crate::Result<()> {
        let (sql, params) = match account_id {
            AccountIdentifier::Id(id) => (
                format!("DELETE FROM {} WHERE key = ?1", self.table_name),
                vec![ToSqlOutput::Owned(Value::Text(id.clone()))],
            ),
            AccountIdentifier::Index(index) => (
                format!(
                    "DELETE FROM {table} WHERE key IN (SELECT key from {table} LIMIT 1 OFFSET {offset})",
                    table = self.table_name,
                    offset = index
                ),
                vec![],
            ),
        };

        let connection = self.connection.lock().expect("failed to get connection lock");
        connection
            .execute(&sql, params)
            .map_err(|_| crate::Error::Storage("failed to delete data".into()))?;
        Ok(())
    }
}

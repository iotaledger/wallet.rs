use super::StorageAdapter;
use crate::account::AccountIdentifier;
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Key value storage adapter.
pub struct SqliteStorageAdapter {
    table_name: String,
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new(db_name: impl AsRef<Path>, table_name: impl AsRef<str>) -> crate::Result<Self> {
        if let Some(folder) = db_name.as_ref().parent() {
            std::fs::create_dir_all(folder)?;
        }
        let connection = Connection::open(db_name)?;

        connection.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    key TEXT NOT NULL UNIQUE,
                    value TEXT
                )",
                table_name.as_ref()
            ),
            NO_PARAMS,
        )?;

        Ok(Self {
            table_name: table_name.as_ref().to_string(),
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

impl StorageAdapter for SqliteStorageAdapter {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let connection = self
            .connection
            .lock()
            .expect("failed to get connection lock");
        let mut query = connection.prepare(&format!(
            "SELECT value FROM {} WHERE key = ?1",
            self.table_name
        ))?;
        let results = query
            .query_and_then(params![id], |row| row.get(0))?
            .collect::<Vec<rusqlite::Result<String>>>();
        let account = results
            .first()
            .map(|val| val.as_ref().unwrap().to_string())
            .ok_or_else(|| anyhow::anyhow!("account isn't stored"))?;
        Ok(account)
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let connection = self
            .connection
            .lock()
            .expect("failed to get connection lock");
        let mut query = connection.prepare(&format!("SELECT value FROM {}", self.table_name))?;
        let accounts = query
            .query_and_then(NO_PARAMS, |row| row.get(0))?
            .map(|val| val.unwrap())
            .collect::<Vec<String>>();
        Ok(accounts)
    }

    fn set(
        &self,
        account_id: AccountIdentifier,
        account: String,
    ) -> std::result::Result<(), anyhow::Error> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let connection = self
            .connection
            .lock()
            .expect("failed to get connection lock");
        let result = connection
            .execute(
                &format!("INSERT OR REPLACE INTO {} VALUES (?1, ?2)", self.table_name),
                params![id, account],
            )
            .map_err(|_| anyhow::anyhow!("failed to insert data"))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> std::result::Result<(), anyhow::Error> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let connection = self
            .connection
            .lock()
            .expect("failed to get connection lock");
        let result = connection
            .execute(
                &format!("DELETE FROM {} WHERE key = ?1", self.table_name),
                params![id],
            )
            .map_err(|_| anyhow::anyhow!("failed to delete data"))?;
        Ok(())
    }
}

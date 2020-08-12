use super::StorageAdapter;
use crate::account::AccountIdentifier;
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Key value storage adapter.
pub struct SqliteStorageAdapter {
  connection: Arc<Mutex<Connection>>,
}

impl SqliteStorageAdapter {
  /// Initialises the storage adapter.
  pub fn new(db_name: impl AsRef<Path>) -> crate::Result<Self> {
    let connection = Connection::open(db_name)?;

    connection.execute(
      "CREATE TABLE IF NOT EXISTS accounts (
          key TEXT NOT NULL UNIQUE,
          value TEXT
        )",
      NO_PARAMS,
    )?;

    Ok(Self {
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
    let mut query = connection.prepare("SELECT value FROM accounts WHERE key = ?1")?;
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
    let mut query = connection.prepare("SELECT value FROM accounts")?;
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
        "INSERT OR REPLACE INTO accounts VALUES (?1, ?2)",
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
      .execute("DELETE FROM accounts WHERE key = ?1", params![id])
      .map_err(|_| anyhow::anyhow!("failed to delete data"))?;
    Ok(())
  }
}

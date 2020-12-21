// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "sqlite")]
/// Sqlite storage.
pub mod sqlite;
#[cfg(feature = "stronghold")]
/// Stronghold storage.
pub mod stronghold;

use crate::account::{Account, AccountIdentifier};
use once_cell::sync::OnceCell;
use tokio::sync::Mutex as AsyncMutex;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

type Storage = Arc<AsyncMutex<Box<dyn StorageAdapter + Sync + Send>>>;
type Storages = Arc<RwLock<HashMap<PathBuf, Storage>>>;
type AccountReadLockMap = HashMap<AccountIdentifier, Arc<Mutex<()>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter<P: AsRef<Path>, S: StorageAdapter + Sync + Send + 'static>(storage_path: P, storage: S) {
    let mut instances = INSTANCES.get_or_init(Default::default).write().unwrap();
    instances.insert(
        storage_path.as_ref().to_path_buf(),
        Arc::new(AsyncMutex::new(Box::new(storage))),
    );
}

pub(crate) fn stronghold_snapshot_filename() -> &'static str {
    "snapshot"
}

/// gets the storage adapter
pub(crate) fn get(storage_path: &PathBuf) -> crate::Result<Storage> {
    let instances = INSTANCES.get_or_init(Default::default).read().unwrap();
    if let Some(instance) = instances.get(storage_path) {
        Ok(instance.clone())
    } else {
        Err(crate::Error::StorageDoesntExist) // TODO proper error kind
    }
}

#[cfg(not(feature = "sqlite"))]
pub(crate) fn get_adapter_from_path<P: AsRef<Path>>(
    storage_path: P,
) -> crate::Result<stronghold::StrongholdStorageAdapter> {
    stronghold::StrongholdStorageAdapter::new(storage_path)
}

#[cfg(feature = "sqlite")]
pub(crate) fn get_adapter_from_path<P: AsRef<Path>>(storage_path: P) -> crate::Result<sqlite::SqliteStorageAdapter> {
    sqlite::SqliteStorageAdapter::new(storage_path, "accounts")
}

/// The storage adapter.
#[async_trait::async_trait]
pub trait StorageAdapter {
    /// Gets the account with the given id/alias from the storage.
    async fn get(&self, account_id: &AccountIdentifier) -> crate::Result<String>;
    /// Gets all the accounts from the storage.
    async fn get_all(&self) -> crate::Result<Vec<String>>;
    /// Saves or updates an account on the storage.
    async fn set(&self, account_id: &AccountIdentifier, account: String) -> crate::Result<()>;
    /// Removes an account from the storage.
    async fn remove(&self, account_id: &AccountIdentifier) -> crate::Result<()>;
}

pub(crate) fn parse_accounts(storage_path: &PathBuf, accounts: &[String]) -> crate::Result<Vec<Account>> {
    let mut err = None;
    let accounts: Vec<Option<Account>> = accounts
        .iter()
        .map(|account| match serde_json::from_str::<Account>(&account) {
            Ok(mut acc) => {
                acc.set_storage_path(storage_path.clone());
                Some(acc)
            }
            Err(e) => {
                err = Some(e);
                None
            }
        })
        .collect();

    if let Some(err) = err {
        Err(err.into())
    } else {
        let accounts = accounts.iter().map(|account| account.clone().unwrap()).collect();
        Ok(accounts)
    }
}

pub(crate) async fn get_account(storage_path: &PathBuf, account_id: &AccountIdentifier) -> crate::Result<Account> {
    let account_str = get(&storage_path)?.lock().await.get(account_id).await?;
    let mut account: Account = serde_json::from_str(&account_str)?;
    account.set_storage_path(storage_path.clone());
    Ok(account)
}

pub(crate) async fn save_account(
    storage_path: &PathBuf,
    account_id: &AccountIdentifier,
    account: String,
) -> crate::Result<()> {
    let storage_handle = get(&storage_path)?;
    let storage = storage_handle.lock().await;
    storage.set(account_id, account).await?;
    Ok(())
}

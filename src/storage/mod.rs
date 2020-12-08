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

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

type Storage = Box<dyn StorageAdapter + Sync + Send>;
type Storages = Arc<RwLock<HashMap<PathBuf, Storage>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter<P: AsRef<Path>, S: StorageAdapter + Sync + Send + 'static>(storage_path: P, storage: S) {
    let mut instances = INSTANCES.get_or_init(Default::default).write().unwrap();
    instances.insert(storage_path.as_ref().to_path_buf(), Box::new(storage));
}

pub(crate) fn stronghold_snapshot_filename() -> &'static str {
    "snapshot"
}

/// gets the storage adapter
pub(crate) fn with_adapter<T, F: FnOnce(&Storage) -> T>(storage_path: &PathBuf, cb: F) -> T {
    let instances = INSTANCES.get_or_init(Default::default).read().unwrap();
    if let Some(instance) = instances.get(storage_path) {
        cb(instance)
    } else {
        panic!(format!("adapter not initialized with path {:?}", storage_path))
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
pub trait StorageAdapter {
    /// Gets the account with the given id/alias from the storage.
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String>;
    /// Gets all the accounts from the storage.
    fn get_all(&self) -> crate::Result<Vec<String>>;
    /// Saves or updates an account on the storage.
    fn set(&self, account_id: AccountIdentifier, account: String) -> crate::Result<()>;
    /// Removes an account from the storage.
    fn remove(&self, account_id: AccountIdentifier) -> crate::Result<()>;
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

pub(crate) fn get_account(storage_path: &PathBuf, account_id: AccountIdentifier) -> crate::Result<Account> {
    let account_str = with_adapter(&storage_path, |storage| storage.get(account_id))?;
    let mut account: Account = serde_json::from_str(&account_str)?;
    account.set_storage_path(storage_path.clone());
    Ok(account)
}

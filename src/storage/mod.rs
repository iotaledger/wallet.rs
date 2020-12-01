// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#[cfg(any(feature = "stronghold", feature = "sqlite"))]
mod sqlite;
#[cfg(feature = "stronghold")]
mod stronghold;

use crate::account::{Account, AccountIdentifier};
use once_cell::sync::OnceCell;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

type Storage = Box<dyn StorageAdapter + Sync + Send>;
type Storages = Arc<Mutex<HashMap<PathBuf, Storage>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter<P: AsRef<Path>, S: StorageAdapter + Sync + Send + 'static>(
    storage_path: P,
    storage: S,
) {
    let mut instances = INSTANCES.get_or_init(Default::default).lock().unwrap();
    instances.insert(storage_path.as_ref().to_path_buf(), Box::new(storage));
}

pub(crate) fn stronghold_snapshot_filename() -> &'static str {
    "snapshot"
}

/// gets the storage adapter
pub(crate) fn with_adapter<T, F: FnOnce(&Storage) -> T>(storage_path: &PathBuf, cb: F) -> T {
    let instances = INSTANCES.get_or_init(Default::default).lock().unwrap();
    if let Some(instance) = instances.get(storage_path) {
        cb(instance)
    } else {
        panic!(format!(
            "adapter not initialized with path {:?}",
            storage_path
        ))
    }
}

#[cfg(not(feature = "sqlite"))]
pub(crate) fn get_adapter_from_path<P: AsRef<Path>>(
    storage_path: P,
) -> crate::Result<stronghold::StrongholdStorageAdapter> {
    stronghold::StrongholdStorageAdapter::new(storage_path)
}

#[cfg(feature = "sqlite")]
pub(crate) fn get_adapter_from_path<P: AsRef<Path>>(
    storage_path: P,
) -> crate::Result<sqlite::SqliteStorageAdapter> {
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

pub(crate) fn parse_accounts(
    storage_path: &PathBuf,
    accounts: &[String],
) -> crate::Result<Vec<Account>> {
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
        let accounts = accounts
            .iter()
            .map(|account| account.clone().unwrap())
            .collect();
        Ok(accounts)
    }
}

pub(crate) fn get_account(
    storage_path: &PathBuf,
    account_id: AccountIdentifier,
) -> crate::Result<Account> {
    let account_str = with_adapter(&storage_path, |storage| storage.get(account_id))?;
    let mut account: Account = serde_json::from_str(&account_str)?;
    account.set_storage_path(storage_path.clone());
    Ok(account)
}

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

pub(crate) fn parse_accounts(accounts: &[String]) -> crate::Result<Vec<Account>> {
    let mut err = None;
    let accounts: Vec<Option<Account>> = accounts
        .iter()
        .map(|account| {
            let res: Option<Account> =
                serde_json::from_str(&account)
                    .map(Some)
                    .unwrap_or_else(|e| {
                        err = Some(e);
                        None
                    });
            res
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
    let account: Account = serde_json::from_str(&account_str)?;
    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::StorageAdapter;
    use crate::account::AccountIdentifier;

    #[test]
    // asserts that the adapter defined by `set_adapter` is globally available with `get_adapter`
    fn set_adapter() {
        struct MyAdapter;
        impl StorageAdapter for MyAdapter {
            fn get(&self, key: AccountIdentifier) -> crate::Result<String> {
                Ok("MY_ADAPTER_GET_RESPONSE".to_string())
            }
            fn get_all(&self) -> crate::Result<Vec<String>> {
                Ok(vec![])
            }
            fn set(&self, key: AccountIdentifier, account: String) -> crate::Result<()> {
                Ok(())
            }
            fn remove(&self, key: AccountIdentifier) -> crate::Result<()> {
                Ok(())
            }
        }

        let path = "./the-storage-path";
        super::set_adapter(path, MyAdapter {});
        super::with_adapter(&std::path::PathBuf::from(path), |adapter| {
            assert_eq!(
                adapter.get([0; 32].into()).unwrap(),
                "MY_ADAPTER_GET_RESPONSE".to_string()
            );
        });
    }

    #[test]
    fn parse_accounts_invalid() {
        let response = super::parse_accounts(&vec!["{}".to_string()]);
        assert!(response.is_err());
    }

    fn _create_account() -> (std::path::PathBuf, crate::account::Account) {
        let manager = crate::test_utils::get_account_manager();

        let client_options =
            crate::client::ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .unwrap()
                .build();
        let account = manager
            .create_account(client_options)
            .alias("alias")
            .initialise()
            .unwrap();
        (manager.storage_path().clone(), account)
    }

    #[test]
    fn parse_accounts_valid() {
        let (_, account) = _create_account();
        let response = super::parse_accounts(&vec![serde_json::to_string(&account).unwrap()]);
        assert!(response.is_ok());
        assert_eq!(response.unwrap().first().unwrap(), &account);
    }

    #[test]
    fn get_account() {
        let (storage_path, account) = _create_account();
        assert_eq!(
            super::get_account(&storage_path, account.id().clone().into()).unwrap(),
            account
        );
    }
}

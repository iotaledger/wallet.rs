mod sqlite;
mod stronghold;

use crate::account::{Account, AccountIdentifier};
use once_cell::sync::OnceCell;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

type Storage = Arc<Mutex<Box<dyn StorageAdapter + Sync + Send>>>;
static INSTANCE: OnceCell<Storage> = OnceCell::new();
static STORAGE_PATH: OnceCell<PathBuf> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter(storage: impl StorageAdapter + Sync + Send + 'static) -> crate::Result<()> {
    INSTANCE
        .set(Arc::new(Mutex::new(Box::new(storage))))
        .map_err(|_| anyhow::anyhow!("failed to globally set the storage instance"))?;
    Ok(())
}

/// Sets the storage path for the default storage adapter.
pub fn set_storage_path(path: impl AsRef<Path>) -> crate::Result<()> {
    STORAGE_PATH
        .set(path.as_ref().to_path_buf())
        .map_err(|_| anyhow::anyhow!("failed to globally set the storage path"))?;
    Ok(())
}

pub(crate) fn get_storage_path() -> &'static PathBuf {
    #[cfg(not(feature = "sqlite"))]
    {
        STORAGE_PATH.get_or_init(|| "./example-database".into())
    }
    #[cfg(feature = "sqlite")]
    {
        STORAGE_PATH.get_or_init(|| "wallet.db".into())
    }
}

/// gets the storage adapter
#[allow(clippy::borrowed_box)]
pub(crate) fn get_adapter(
) -> crate::Result<MutexGuard<'static, Box<dyn StorageAdapter + Sync + Send>>> {
    let instance: crate::Result<&Storage> = INSTANCE.get_or_try_init(|| {
        let storage_path = get_storage_path();
        let instance =
            Arc::new(Mutex::new(Box::new(get_adapter_from_path(storage_path)?)
                as Box<dyn StorageAdapter + Sync + Send>));
        Ok(instance)
    });
    Ok(instance?.lock().unwrap())
}

#[cfg(not(feature = "sqlite"))]
pub(crate) fn get_adapter_from_path<'a, P: AsRef<Path>>(
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
    fn get(&self, key: AccountIdentifier) -> crate::Result<String>;
    /// Gets all the accounts from the storage.
    fn get_all(&self) -> crate::Result<Vec<String>>;
    /// Saves or updates an account on the storage.
    fn set(&self, key: AccountIdentifier, account: String) -> crate::Result<()>;
    /// Removes an account from the storage.
    fn remove(&self, key: AccountIdentifier) -> crate::Result<()>;
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

pub(crate) fn get_account(account_id: AccountIdentifier) -> crate::Result<Account> {
    let adapter = crate::storage::get_adapter()?;
    let account_str = adapter.get(account_id)?;
    let account: Account = serde_json::from_str(&account_str)?;
    Ok(account)
}

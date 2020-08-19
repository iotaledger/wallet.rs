#[cfg(not(any(feature = "sqlite", feature = "stronghold")))]
mod key_value;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "stronghold")]
mod stronghold;

use crate::account::{Account, AccountIdentifier};
use crate::address::Address;
use crate::transaction::Transaction;
use bee_crypto::ternary::Hash;
use once_cell::sync::OnceCell;

use std::path::{Path, PathBuf};

static INSTANCE: OnceCell<Box<dyn StorageAdapter + Sync + Send>> = OnceCell::new();
static STORAGE_PATH: OnceCell<PathBuf> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter(storage: impl StorageAdapter + Sync + Send + 'static) -> crate::Result<()> {
  INSTANCE
    .set(Box::new(storage))
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

/// gets the storage adapter
#[allow(clippy::borrowed_box)]
pub(crate) fn get_adapter() -> crate::Result<&'static Box<dyn StorageAdapter + Sync + Send>> {
  INSTANCE.get_or_try_init(|| {
    #[cfg(not(any(feature = "sqlite", feature = "stronghold")))]
    {
      let storage_path = STORAGE_PATH.get_or_init(|| "./example-database".into());
      let instance = Box::new(key_value::KeyValueStorageAdapter::new(storage_path)?)
        as Box<dyn StorageAdapter + Sync + Send>;
      Ok(instance)
    }
    #[cfg(feature = "stronghold")]
    {
      let storage_path = STORAGE_PATH.get_or_init(|| "./example-database".into());
      let instance = Box::new(stronghold::StrongholdStorageAdapter::new(storage_path)?)
        as Box<dyn StorageAdapter + Sync + Send>;
      Ok(instance)
    }
    #[cfg(feature = "sqlite")]
    {
      let storage_path = STORAGE_PATH.get_or_init(|| "wallet.db".into());
      let instance = Box::new(sqlite::SqliteStorageAdapter::new(storage_path)?)
        as Box<dyn StorageAdapter + Sync + Send>;
      Ok(instance)
    }
  })
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

pub(crate) fn parse_accounts(accounts: &Vec<String>) -> crate::Result<Vec<Account>> {
  let mut err = None;
  let accounts: Vec<Option<Account>> = accounts
    .iter()
    .map(|account| {
      let res: Option<Account> = serde_json::from_str(&account)
        .map(|v| Some(v))
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

/// Gets the account's total balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
pub(crate) fn total_balance(account_id: String) -> crate::Result<u64> {
  unimplemented!()
}

/// Gets the account's available balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
///
/// The available balance is the balance users are allowed to spend.
/// For example, if a user with 50i total account balance has made a transaction spending 20i,
/// the available balance should be (50i-30i) = 20i.
pub(crate) fn available_balance(account_id: String) -> crate::Result<u64> {
  unimplemented!()
}

/// Gets the transaction associated with the given hash.
pub(crate) fn get_transaction(transaction_hash: Hash) -> crate::Result<Transaction> {
  unimplemented!()
}

/// Gets a new unused address and links it to the given account.
pub(crate) fn save_address(account_id: String, address: &Address) -> crate::Result<Address> {
  unimplemented!()
}

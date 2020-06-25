use super::{Account, AccountInitialiser, SyncedAccount};
use crate::storage::{MemoryStorageAdapter, StorageAdapter};
use chrono::prelude::Utc;
use std::marker::PhantomData;
use std::path::Path;

/// The account manager.
///
/// Used to manage multiple accounts.
pub struct AccountManager<'a, T: StorageAdapter> {
  storage_adapter: T,
  _phantom: &'a PhantomData<T>,
}

impl<'a> Default for AccountManager<'a, MemoryStorageAdapter> {
  fn default() -> Self {
    Self {
      storage_adapter: MemoryStorageAdapter::new(),
      _phantom: &PhantomData,
    }
  }
}

impl<'a> AccountManager<'a, MemoryStorageAdapter> {
  /// Initialises a new instance of the account manager with the default storage adapter.
  pub fn new() -> Self {
    Default::default()
  }
}

impl<'a, T: StorageAdapter + Clone> AccountManager<'a, T> {
  /// Initialises a new instance of the account manager with the given storage adapter.
  pub fn with_adapter(adapter: T) -> Self {
    Self {
      storage_adapter: adapter,
      _phantom: &PhantomData,
    }
  }

  /// Adds a new account.
  pub fn add_account(&mut self, account: &AccountInitialiser<'a>) -> crate::Result<Account<'a, T>> {
    let alias = account.alias();
    // crate::account::init(&account)?;
    self
      .storage_adapter
      .set(alias, serde_json::to_string(&account)?)?;
    Ok(Account {
      storage_adapter: self.storage_adapter.clone(),
      alias,
      nodes: vec![],
      quorum_size: None,
      quorum_threshold: None,
      network: None,
      provider: None,
      created_at: Utc::now(),
      transactions: vec![],
      addresses: vec![],
    })
  }

  /// Deletes an account.
  pub fn remove_account(&mut self, account_id: &str) -> crate::Result<()> {
    self.storage_adapter.remove(account_id)
  }

  /// Syncs all accounts.
  pub fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
    unimplemented!()
  }

  /// Transfers an amount from an account to another.
  pub fn transfer(
    &self,
    from_account_id: &str,
    to_account_id: &str,
    amount: f64,
  ) -> crate::Result<()> {
    unimplemented!()
  }

  /// Backups the accounts to the given destination
  pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<()> {
    unimplemented!()
  }

  /// Gets the account associated with the given address.
  pub fn get_account_from_address(address: &str) -> crate::Result<Account<'a, T>> {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::AccountManager;
  use crate::account::AccountInitialiserBuilder;

  #[test]
  fn store_accounts() {
    let mut manager = AccountManager::new();
    let alias = "test";
    let account = AccountInitialiserBuilder::new()
      .alias(alias)
      .nodes(vec!["https://nodes.devnet.iota.org:443"])
      .build()
      .expect("failed to build account");

    manager
      .add_account(&account)
      .expect("failed to add account");
    manager
      .remove_account(alias)
      .expect("failed to remove account");
  }
}

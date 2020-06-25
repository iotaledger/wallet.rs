use super::{Account, SyncedAccount};
use crate::storage::{MemoryStorageAdapter, StorageAdapter};
use std::path::Path;

/// The account manager.
///
/// Used to manage multiple accounts.
pub struct AccountManager<'a> {
  storage_adapter: Box<dyn StorageAdapter<'a>>,
}

impl<'a> AccountManager<'a> {
  // TODO this doesn't compile
  /*/// Initialises a new instance of the account manager with the default storage adapter.
  pub fn new() -> Self {
    Self {
      storage_adapter: Box::new(MemoryStorageAdapter::new()),
    }
  }*/

  /// Initialises a new instance of the account manager with the given storage adapter.
  pub fn with_adapter(adapter: Box<dyn StorageAdapter<'a>>) -> Self {
    Self {
      storage_adapter: adapter,
    }
  }

  /// Adds a new account.
  pub fn add_account(&mut self, account: Account<'a>) -> crate::Result<()> {
    (*self.storage_adapter).set(account.alias(), account)
  }

  /// Deletes an account.
  pub fn remove_account(&mut self, account_id: &str) -> crate::Result<()> {
    (*self.storage_adapter).remove(account_id)
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
  pub fn get_account_from_address(address: &str) -> crate::Result<Account<'a>> {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::AccountManager;
  use crate::account::AccountBuilder;
  use crate::storage::MemoryStorageAdapter;

  #[test]
  fn store_accounts() {
    let mut manager = AccountManager::with_adapter(Box::new(MemoryStorageAdapter::new()));
    let alias = "test";
    let account = AccountBuilder::new()
      .alias(alias)
      .nodes(vec!["https://nodes.devnet.iota.org:443"])
      .build()
      .expect("failed to build account");

    manager.add_account(account).expect("failed to add account");
    manager
      .remove_account(alias)
      .expect("failed to remove account");
  }
}

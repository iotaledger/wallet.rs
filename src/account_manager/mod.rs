mod account;

use crate::address::Address;
use crate::transaction::Transfer;
use account::{Account, AccountInitialiser};
use std::path::Path;

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Default)]
pub struct AccountManager {}

impl<'a> AccountManager {
  /// Initialises a new instance of the account manager with the default storage adapter.
  pub fn new() -> Self {
    Default::default()
  }

  /// Adds a new account.
  pub fn create_account(&mut self) -> AccountInitialiser<'a> {
    AccountInitialiser::new()
  }

  /// Deletes an account.
  pub fn remove_account(&mut self, account_id: &str) -> crate::Result<()> {
    crate::storage::get_adapter()?.remove(account_id)
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
    amount: u64,
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

/// Data returned from account synchronization.
pub struct SyncedAccount {
  deposit_address: Address,
}

impl SyncedAccount {
  /// The account's deposit address.
  pub fn deposit_address(&self) -> &Address {
    &self.deposit_address
  }

  /// Send transactions.
  pub fn send<'a>(&self, transfers: Vec<Transfer<'a>>) {}

  /// Retry transactions.
  pub fn retry(&self) {}
}

#[cfg(test)]
mod tests {
  use super::AccountManager;

  #[test]
  fn store_accounts() {
    let mut manager = AccountManager::new();
    let id = "test";

    manager
      .create_account()
      .alias(id)
      .id(id)
      .mnemonic(id)
      .nodes(vec!["https://nodes.devnet.iota.org:443"])
      .initialise()
      .expect("failed to add account");

    manager
      .remove_account(id)
      .expect("failed to remove account");
  }
}

use super::{Account, AccountInitialiser, SyncedAccount};
use chrono::prelude::Utc;
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
  pub fn add_account(&mut self, account: &AccountInitialiser<'a>) -> crate::Result<Account<'a>> {
    let id = account.id();
    // crate::account::init(&account)?;
    crate::storage::get_adapter()?.set(id, serde_json::to_string(&account)?)?;
    Ok(Account {
      id,
      alias: account.alias(),
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
  use crate::account::AccountInitialiserBuilder;

  #[test]
  fn store_accounts() {
    let mut manager = AccountManager::new();
    let id = "test";
    let account = AccountInitialiserBuilder::new()
      .alias(id)
      .id(id)
      .mnemonic(id)
      .nodes(vec!["https://nodes.devnet.iota.org:443"])
      .build()
      .expect("failed to build account");

    manager
      .add_account(&account)
      .expect("failed to add account");
    manager
      .remove_account(id)
      .expect("failed to remove account");
  }
}

mod account;
mod api;

use crate::address::Address;
use account::{Account, AccountInitialiser};
use api::SyncedAccount;
use std::path::Path;

/// An acccount identifier.
pub struct AccountIdentifier<T> {
  value: T,
}

// When the identifier is an Address.
impl From<Address> for AccountIdentifier<Address> {
  fn from(value: Address) -> Self {
    Self { value }
  }
}

// When the identifier is a String (alias).
impl<S: Into<String>> From<S> for AccountIdentifier<String> {
  fn from(value: S) -> Self {
    Self {
      value: value.into(),
    }
  }
}

// When the identifier is an id.
impl<S: Into<u64>> From<S> for AccountIdentifier<u64> {
  fn from(value: S) -> Self {
    Self {
      value: value.into(),
    }
  }
}

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
  pub fn create_account(&self) -> AccountInitialiser<'a> {
    AccountInitialiser::new()
  }

  /// Deletes an account.
  pub fn remove_account<T>(&self, account_id: AccountIdentifier<T>) -> crate::Result<()> {
    let account = self.get_account(account_id)?;
    crate::storage::get_adapter()?.remove(account.id())
  }

  /// Syncs all accounts.
  pub fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
    unimplemented!()
  }

  /// Transfers an amount from an account to another.
  pub fn transfer<F, T>(
    &self,
    from_account_id: AccountIdentifier<F>,
    to_account_id: AccountIdentifier<T>,
    amount: u64,
  ) -> crate::Result<()> {
    unimplemented!()
  }

  /// Backups the accounts to the given destination
  pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<()> {
    unimplemented!()
  }

  /// Import backed up accounts.
  pub fn import_accounts(&self, accounts: Vec<Account<'a>>) -> crate::Result<()> {
    unimplemented!()
  }

  /// Gets the account associated with the given identifier.
  pub fn get_account<T>(&self, account_id: AccountIdentifier<T>) -> crate::Result<Account<'a>> {
    let account = Account::new("test");
    Ok(account)
  }

  /// Reattaches an unconfirmed transaction.
  pub fn reattach<T>(&self, account_id: AccountIdentifier<T>) -> crate::Result<()> {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::AccountManager;

  #[test]
  fn store_accounts() {
    let manager = AccountManager::new();
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
      .remove_account(id.into())
      .expect("failed to remove account");
  }
}

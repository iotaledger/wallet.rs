mod api;

use crate::account::{Account, AccountIdentifier, AccountInitialiser};
use crate::client::ClientOptions;
use api::{AccountSynchronizer, SyncedAccount};
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
  pub fn create_account(&self, client_options: ClientOptions) -> AccountInitialiser<'a> {
    AccountInitialiser::new(client_options)
  }

  /// Deletes an account.
  pub fn remove_account(&self, account_id: AccountIdentifier) -> crate::Result<()> {
    crate::storage::get_adapter()?.remove(account_id)
  }

  /// Syncs all accounts.
  pub fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    let mut synced_accounts = vec![];
    for account_str in accounts {
      let account: Account<'_> = serde_json::from_str(&account_str)?;
      let synced_account = AccountSynchronizer::new(&account).execute()?;
      synced_accounts.push(synced_account);
    }
    Ok(synced_accounts)
  }

  /// Transfers an amount from an account to another.
  pub fn transfer(
    &self,
    from_account_id: AccountIdentifier,
    to_account_id: AccountIdentifier,
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
  pub fn get_account(&self, account_id: AccountIdentifier) -> crate::Result<Account<'a>> {
    let account_str = crate::storage::get_adapter()?.get(account_id)?;
    // serde_json::from_str(&account_str).map_err(|e| e.into());
    unimplemented!()
  }

  /// Reattaches an unconfirmed transaction.
  pub fn reattach<T>(&self, account_id: AccountIdentifier) -> crate::Result<()> {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::AccountManager;
  use crate::client::ClientOptionsBuilder;

  #[test]
  fn store_accounts() {
    let manager = AccountManager::new();
    let id = "test";
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
      .expect("invalid node URL")
      .build();

    manager
      .create_account(client_options)
      .alias(id)
      .id(id)
      .mnemonic(id)
      // TODO .nodes(vec!["https://nodes.devnet.iota.org:443"])
      .initialise()
      .expect("failed to add account");

    manager
      .remove_account(id.to_string().into())
      .expect("failed to remove account");
  }
}

use crate::address::Address;
use crate::transaction::Transfer;

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

/// Syncs account with the tangle.
/// Gets the latest balance for the account
/// and finds new transactions associated with it.
pub(crate) fn sync(account_id: &str) -> crate::Result<SyncedAccount> {
  unimplemented!()
}

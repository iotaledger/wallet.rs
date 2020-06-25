mod memory;

pub use memory::MemoryStorageAdapter;

use crate::address::Address;
use crate::transaction::Transaction;

/// The storage adapter.
pub trait StorageAdapter {
  /// Gets the account with the given id/alias from the storage.
  fn get(&mut self, key: &str) -> crate::Result<&String>;
  /// Gets all the accounts from the storage.
  fn get_all(&mut self) -> crate::Result<Vec<&String>>;
  /// Saves or updates an account on the storage.
  fn set(&mut self, key: &str, account: String) -> crate::Result<()>;
  /// Removes an account from the storage.
  fn remove(&mut self, key: &str) -> crate::Result<()>;
}

/// Transaction type.
pub enum TransactionType {
  /// Transaction received.
  Received,
  /// Transaction sent.
  Sent,
}

/// Gets the account's total balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
pub(crate) fn total_balance(
  storage: &mut impl StorageAdapter,
  account_id: &str,
) -> crate::Result<f64> {
  unimplemented!()
}

/// Gets the account's available balance.
/// It's read directly from the storage. To read the latest account balance, you should `sync` first.
///
/// The available balance is the balance users are allowed to spend.
/// For example, if a user with 50i total account balance has made a transaction spending 20i,
/// the available balance should be (50i-30i) = 20i.
pub(crate) fn available_balance(
  storage: &mut impl StorageAdapter,
  account_id: &str,
) -> crate::Result<f64> {
  unimplemented!()
}

/// Updates the account alias.
pub(crate) fn set_alias(
  storage: &mut impl StorageAdapter,
  account_id: &str,
  alias: &str,
) -> crate::Result<()> {
  unimplemented!()
}

/// Gets a list of transactions on the given account.
/// It's fetched from the storage. To ensure the database is updated with the latest transactions,
/// `sync` should be called first.
///
/// * `account_id` - The account identifier
/// * `count` - Number of (most recent) transactions to fetch.
/// * `from` - Starting point of the subset to fetch.
/// * `transaction_type` - Optional transaction type filter.
pub(crate) fn list_transactions<'a>(
  storage: &mut impl StorageAdapter,
  account_id: &str,
  count: u64,
  from: u64,
  transaction_type: Option<TransactionType>,
) -> crate::Result<Vec<Transaction<'a>>> {
  Ok(vec![])
}

/// Gets the addresses linked to the given account.
///
/// * `account_id` - The account identifier.
/// * `unspent` - Whether it should get only unspent addresses or not.
pub(crate) fn list_addresses(
  storage: &mut impl StorageAdapter,
  account_id: &str,
  unspent: bool,
) -> crate::Result<Vec<Address>> {
  unimplemented!()
}

/// Gets a new unused address and links it to the given account.
pub(crate) fn generate_address(
  storage: &mut impl StorageAdapter,
  account_id: &str,
) -> crate::Result<Address> {
  unimplemented!()
}

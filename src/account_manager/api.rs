use crate::address::Address;
use crate::transaction::{Transaction, Transfer};
use bee_crypto::ternary::Hash;

/// Syncs addresses with the tangle.
/// The method ensures that the wallet local state has all used addresses plus an unused address.
///
/// To sync addresses for an account from scratch, `address_index` = 0 and `gap_limit` = 20 should be provided.
/// To sync addresses from the latest address, `address_index` = latest address index and `gap_limit` = 1 should be provided.
///
/// # Arguments
///
/// * `address_index` The address index.
/// * `gap_limit` Number of addresses indexes that are generated.
///
/// # Return value
///
/// Returns a (addresses, hashes) tuples representing the address history up to latest unused address,
/// and the transaction hashes associated with the addresses.
///
fn sync_addresses(
  address_index: u64,
  gap_limit: Option<u64>,
) -> crate::Result<(Vec<Address>, Vec<Hash>)> {
  unimplemented!()
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
fn sync_transactions<'a>(new_transaction_hashes: Vec<Hash>) -> crate::Result<Vec<Transaction<'a>>> {
  unimplemented!()
}

/// Account sync helper.
pub(super) struct AccountSynchronizer<'a> {
  account_id: &'a str,
  address_index: u64,
  gap_limit: Option<u64>,
  skip_persistance: bool,
}

impl<'a> AccountSynchronizer<'a> {
  /// Initialises a new instance of the sync helper.
  pub fn new(account_id: &'a str) -> Self {
    Self {
      account_id,
      address_index: 1, // TODO By default the length of addresses stored for this account should be used as an index.
      gap_limit: None,
      skip_persistance: false,
    }
  }

  /// Sets the address index.
  /// By default the length of addresses stored for this account should be used as an index.
  pub fn address_index(mut self, index: u64) -> Self {
    self.address_index = index;
    self
  }

  /// Number of address indexes that are generated.
  pub fn gap_limit(mut self, limit: u64) -> Self {
    self.gap_limit = Some(limit);
    self
  }

  /// Skip write to the database.
  pub fn skip_persistance(mut self) -> Self {
    self.skip_persistance = true;
    self
  }

  /// Syncs account with the tangle.
  /// The account syncing process ensures that the latest metadata (balance, transactions)
  /// associated with an account is fetched from the tangle and is stored locally.
  pub fn sync(self) -> crate::Result<SyncedAccount> {
    sync_addresses(self.address_index, self.gap_limit)?;
    sync_transactions(vec![])?;
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
  pub fn transfer<'a>(&self, transfer_obj: Transfer<'a>) -> crate::Result<Transaction<'a>> {
    transfer(transfer_obj)
  }

  /// Retry transactions.
  pub fn retry<'a>(&self) -> crate::Result<Transaction<'a>> {
    retry(Hash::zeros())
  }
}

/// Starts the account sync process.
pub(super) fn sync<'a>(account_id: &'a str) -> AccountSynchronizer<'a> {
  AccountSynchronizer::new(account_id)
}

/// Selects input addresses for a value transaction.
/// The method ensures that the recipient address doesn’t match any of the selected inputs or the remainder address.
///
/// # Arguments
///
/// * `threshold` Amount user wants to spend.
/// * `address` Recipient address.
///
/// # Return value
///
/// Returns a (addresses, address) tuple representing the selected input addresses and the remainder address if needed.
fn select_inputs(
  threshold: u64,
  address: &Address,
) -> crate::Result<(Vec<Address>, Option<Address>)> {
  unimplemented!()
}

/// Sends a value transaction to the tangle.
pub(super) fn transfer<'a>(transfer: Transfer<'a>) -> crate::Result<Transaction<'a>> {
  select_inputs(*transfer.amount(), transfer.address())?;
  unimplemented!()
}

pub(super) fn send_message<'a>(transfer: Transfer<'a>) -> crate::Result<Transaction<'a>> {
  unimplemented!()
}

/// Rebroadcasts a failed transaction.
pub(super) fn retry<'a>(transaction_hash: Hash) -> crate::Result<Transaction<'a>> {
  let transaction = crate::storage::get_transaction(transaction_hash)?;
  unimplemented!()
}

pub(super) fn reattach<'a>(transaction_hash: Hash) -> crate::Result<Transaction<'a>> {
  unimplemented!()
}

use crate::account::Account;
use crate::address::{Address, AddressBuilder, IotaAddress};
use crate::client::{with_client, ClientOptions};
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
fn sync_addresses<'a>(
  account: &'a Account<'_>,
  address_index: u64,
  gap_limit: Option<u64>,
) -> crate::Result<(Vec<Address>, Vec<Hash>)> {
  let addresses = account.addresses();
  let transactions = account.transactions();
  let latest_address = account.latest_address();
  with_client(account.client_options(), |client| {
    for transaction in transactions {}
    for address in addresses {}
    // TODO add seed here
    // client.balance();
  });
  unimplemented!()
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
fn sync_transactions<'a>(
  account: &'a Account<'_>,
  new_transaction_hashes: Vec<Hash>,
) -> crate::Result<Vec<Transaction>> {
  with_client(account.client_options(), |client| {
    for address in account.addresses() {
      // TODO: implement this when iota.rs and wallet.rs uses the same bee-transaction
      /*client
      .find_transactions()
      .addresses(&[address.address().clone()]);*/
    }
  });
  unimplemented!()
}

/// Account sync helper.
pub struct AccountSynchronizer<'a> {
  account: &'a Account<'a>,
  address_index: u64,
  gap_limit: Option<u64>,
  skip_persistance: bool,
}

impl<'a> AccountSynchronizer<'a> {
  /// Initialises a new instance of the sync helper.
  pub fn new(account: &'a Account<'_>) -> Self {
    Self {
      account,
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
  pub fn execute(self) -> crate::Result<SyncedAccount> {
    sync_addresses(self.account, self.address_index, self.gap_limit)?;
    sync_transactions(self.account, vec![])?;

    let synced_account = SyncedAccount {
      client_options: self.account.client_options().clone(),
      deposit_address: AddressBuilder::new()
        .address(IotaAddress::zeros())
        .balance(0)
        .key_index(0)
        .build()?,
    };
    Ok(synced_account)
  }
}

/// Data returned from account synchronization.
pub struct SyncedAccount {
  client_options: ClientOptions,
  deposit_address: Address,
}

impl SyncedAccount {
  /// The account's deposit address.
  pub fn deposit_address(&self) -> &Address {
    &self.deposit_address
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
    &self,
    threshold: &u64,
    address: &Address,
  ) -> crate::Result<(Vec<Address>, Option<Address>)> {
    unimplemented!()
  }

  /// Send transactions.
  pub fn transfer(&self, transfer_obj: Transfer) -> crate::Result<Transaction> {
    self.select_inputs(transfer_obj.amount(), transfer_obj.address())?;
    unimplemented!()
  }

  /// Retry transactions.
  pub fn retry(&self, transaction_hash: Hash) -> crate::Result<Transaction> {
    let transaction = crate::storage::get_transaction(transaction_hash)?;
    unimplemented!()
  }
}

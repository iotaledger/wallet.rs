use crate::address::{Address, AddressBuilder, IotaAddress};
use crate::client::with_client;
use crate::transaction::{Transaction, Transfer};
use bee_crypto::ternary::Hash;
use iota::Client;

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
  client: &'a Client,
  address_index: u64,
  gap_limit: Option<u64>,
) -> crate::Result<(Vec<Address>, Vec<Hash>)> {
  let accounts = crate::storage::get_adapter()?.get_all()?;
  let accounts = crate::storage::parse_accounts(&accounts)?;

  for account in accounts {
    let addresses = account.addresses();
    let transactions = account.transactions();
    let latest_address = account.latest_address();
    with_client(account.client_options(), |client| {
      for transaction in transactions {}
      for address in addresses {}
      // TODO add seed here
      // client.get_balance();
    })
  }

  unimplemented!()
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
fn sync_transactions<'a>(
  client: &'a Client,
  new_transaction_hashes: Vec<Hash>,
) -> crate::Result<Vec<Transaction>> {
  client.find_transactions();
  unimplemented!()
}

/// The high level client interface wrapper.
pub struct ApiClient<'a> {
  client: &'a Client,
}

impl<'a> ApiClient<'a> {
  /// Initialises a new instance of the ApiClient.
  pub fn new(client: &'a Client) -> Self {
    Self { client }
  }

  /// Starts the account sync process.
  pub fn sync(&self, account_id: &'a str) -> AccountSynchronizer<'a> {
    AccountSynchronizer::new(account_id, &self.client)
  }

  pub fn send_message(self, transfer: Transfer) -> crate::Result<Transaction> {
    unimplemented!()
  }

  pub fn reattach(self, transaction_hash: Hash) -> crate::Result<Transaction> {
    unimplemented!()
  }
}

/// Account sync helper.
pub struct AccountSynchronizer<'a> {
  account_id: &'a str,
  address_index: u64,
  gap_limit: Option<u64>,
  skip_persistance: bool,
  client: &'a Client,
}

impl<'a> AccountSynchronizer<'a> {
  /// Initialises a new instance of the sync helper.
  pub fn new(account_id: &'a str, client: &'a Client) -> Self {
    Self {
      account_id,
      address_index: 1, // TODO By default the length of addresses stored for this account should be used as an index.
      gap_limit: None,
      skip_persistance: false,
      client,
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
  pub fn sync(self) -> crate::Result<SyncedAccount<'a>> {
    sync_addresses(&self.client, self.address_index, self.gap_limit)?;
    sync_transactions(&self.client, vec![])?;

    let synced_account = SyncedAccount {
      client: &self.client,
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
pub struct SyncedAccount<'a> {
  client: &'a Client,
  deposit_address: Address,
}

impl<'a> SyncedAccount<'a> {
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

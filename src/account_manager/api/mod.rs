use crate::account::Account;
use crate::address::{Address, AddressBuilder, IotaAddress};
use crate::client::{get_client, ClientOptions};
use crate::transaction::{Transaction, Transfer};
use bee_crypto::ternary::Hash;

mod input_selection;

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
  let client = get_client(account.client_options());
  for transaction in transactions {}
  for address in addresses {}
  // TODO add seed here
  // client.balance();
  unimplemented!()
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
async fn sync_transactions<'a>(
  account: &'a Account<'_>,
  new_transaction_hashes: Vec<Hash>,
) -> crate::Result<Vec<Transaction>> {
  let mut transactions: Vec<Transaction> = account.transactions().iter().cloned().collect();

  // sync `broadcasted` state
  transactions
    .iter_mut()
    .filter(|tx| !tx.broadcasted() && new_transaction_hashes.contains(tx.hash()))
    .for_each(|tx| {
      tx.set_broadcasted(true);
    });

  // sync `confirmed` state
  let mut unconfirmed_transactions: Vec<&mut Transaction> = transactions
    .iter_mut()
    .filter(|tx| !tx.confirmed())
    .collect();
  let client = get_client(account.client_options());
  let unconfirmed_transaction_hashes: Vec<Hash> = unconfirmed_transactions
    .iter()
    .map(|tx| tx.hash().clone())
    .collect();
  let confirmed_states = client
    .is_confirmed(&unconfirmed_transaction_hashes[..])
    .await
    .unwrap();
  for (tx, confirmed) in unconfirmed_transactions
    .iter_mut()
    .zip(confirmed_states.iter())
  {
    if *confirmed {
      tx.set_confirmed(true);
    }
  }

  // get new transactions
  let response = client
    .get_trytes(&new_transaction_hashes[..])
    .await
    .unwrap();
  let mut hashes_iter = new_transaction_hashes.iter();

  for tx in response.trytes {
    let hash = hashes_iter.next().unwrap();
    transactions.push(Transaction::from_bundled(*hash, tx).unwrap());
  }

  Ok(transactions)
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
  pub async fn execute(self) -> crate::Result<SyncedAccount> {
    let client = get_client(self.account.client_options());
    let mut addresses = vec![];
    for address in self.account.addresses() {
      addresses.push(address.address().clone());
    }

    let mut new_transaction_hashes = vec![];
    let find_transactions_response = client
      .find_transactions()
      .addresses(&addresses[..])
      .send()
      .await
      .unwrap();
    for tx_hash in find_transactions_response.hashes {
      if !self
        .account
        .transactions()
        .iter()
        .any(|tx| tx.hash() == &tx_hash)
      {
        new_transaction_hashes.push(tx_hash);
      }
    }

    sync_addresses(self.account, self.address_index, self.gap_limit)?;
    sync_transactions(self.account, new_transaction_hashes).await?;

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

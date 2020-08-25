use crate::address::Address;
use crate::client::ClientOptions;
use crate::transaction::{Transaction, TransactionType};

use bee_crypto::ternary::Hash;
use bee_signing::ternary::seed::Seed;
use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

mod sync;
pub use sync::{AccountSynchronizer, SyncedAccount};

/// The account identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountIdentifier {
  /// An Id (string) identifier.
  Id(String),
  /// An index identifier.
  Index(u64),
}

// When the identifier is a String (id).
impl From<String> for AccountIdentifier {
  fn from(value: String) -> Self {
    Self::Id(value)
  }
}

// When the identifier is an id.
impl From<u64> for AccountIdentifier {
  fn from(value: u64) -> Self {
    Self::Index(value)
  }
}

/// Account initialiser.
pub struct AccountInitialiser {
  mnemonic: Option<String>,
  id: Option<String>,
  alias: Option<String>,
  created_at: Option<DateTime<Utc>>,
  transactions: Vec<Transaction>,
  addresses: Vec<Address>,
  client_options: ClientOptions,
}

impl AccountInitialiser {
  /// Initialises the account builder.
  pub(crate) fn new(client_options: ClientOptions) -> Self {
    Self {
      mnemonic: None,
      id: None,
      alias: None,
      created_at: None,
      transactions: vec![],
      addresses: vec![],
      client_options,
    }
  }

  /// Defines the account BIP-39 mnemonic.
  /// When importing an account from stronghold, the mnemonic won't be required.
  pub fn mnemonic(mut self, mnemonic: impl AsRef<str>) -> Self {
    self.mnemonic = Some(mnemonic.as_ref().to_string());
    self
  }

  /// SHA-256 hash of the first address on the seed.
  /// Required for referencing a seed in stronghold.
  /// The id should be provided by stronghold.
  pub fn id(mut self, id: impl AsRef<str>) -> Self {
    self.id = Some(id.as_ref().to_string());
    self
  }

  /// Defines the account alias. If not defined, we'll generate one.
  pub fn alias(mut self, alias: impl AsRef<str>) -> Self {
    self.alias = Some(alias.as_ref().to_string());
    self
  }

  /// Time of account creation.
  pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
    self.created_at = Some(created_at);
    self
  }

  /// Transactions associated with the seed.
  /// The account can be initialised with locally stored transactions.
  pub fn transactions(mut self, transactions: Vec<Transaction>) -> Self {
    self.transactions = transactions;
    self
  }

  // Address history associated with the seed.
  /// The account can be initialised with locally stored address history.
  pub fn addresses(mut self, addresses: Vec<Address>) -> Self {
    self.addresses = addresses;
    self
  }

  /// Initialises the account.
  pub fn initialise(self) -> crate::Result<Account> {
    let alias = self.alias.unwrap_or_else(|| "".to_string());
    let id = self.id.unwrap_or(alias.clone());
    let account_id: AccountIdentifier = id.to_string().into();

    let account = Account {
      id,
      alias,
      created_at: self.created_at.unwrap_or_else(chrono::Utc::now),
      transactions: self.transactions,
      addresses: self.addresses,
      client_options: self.client_options,
    };
    let adapter = crate::storage::get_adapter()?;
    adapter.set(account_id, serde_json::to_string(&account)?)?;
    Ok(account)
  }
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct Account {
  /// The account identifier.
  id: String,
  /// The account alias.
  alias: String,
  /// Time of account creation.
  created_at: DateTime<Utc>,
  /// Transactions associated with the seed.
  /// The account can be initialised with locally stored transactions.
  #[serde(skip)]
  #[getset(set = "pub(crate)")]
  transactions: Vec<Transaction>,
  /// Address history associated with the seed.
  /// The account can be initialised with locally stored address history.
  #[serde(skip)]
  addresses: Vec<Address>,
  /// The client options.
  client_options: ClientOptions,
}

impl Account {
  pub(crate) fn latest_address(&self) -> &Address {
    &self.addresses.iter().max_by_key(|a| a.key_index()).unwrap()
  }

  pub(crate) fn seed(&self) -> &Seed {
    unimplemented!()
  }

  /// Returns the builder to setup the process to synchronize this account with the Tangle.
  pub fn sync(&self) -> AccountSynchronizer<'_> {
    AccountSynchronizer::new(self)
  }

  /// Gets the account's total balance.
  /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
  pub fn total_balance(&mut self) -> u64 {
    self
      .addresses
      .iter()
      .fold(0, |acc, address| acc + address.balance())
  }

  /// Gets the account's available balance.
  /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
  ///
  /// The available balance is the balance users are allowed to spend.
  /// For example, if a user with 50i total account balance has made a transaction spending 20i,
  /// the available balance should be (50i-30i) = 20i.
  pub fn available_balance(&mut self) -> u64 {
    let total_balance = self.total_balance();
    let spent = self.transactions.iter().fold(0, |acc, tx| {
      let val = if *tx.confirmed() {
        0
      } else {
        tx.value().without_denomination()
      };
      acc + val
    });
    total_balance - (spent as u64)
  }

  /// Updates the account alias.
  pub fn set_alias(&mut self, alias: impl AsRef<str>) -> crate::Result<()> {
    self.alias = alias.as_ref().to_string();
    crate::storage::get_adapter()?.set(self.id.to_string().into(), serde_json::to_string(self)?)?;
    Ok(())
  }

  /// Gets a list of transactions on this account.
  /// It's fetched from the storage. To ensure the database is updated with the latest transactions,
  /// `sync` should be called first.
  ///
  /// * `count` - Number of (most recent) transactions to fetch.
  /// * `from` - Starting point of the subset to fetch.
  /// * `transaction_type` - Optional transaction type filter.
  ///
  /// # Example
  ///
  /// ```
  /// use iota_wallet::transaction::TransactionType;
  /// use iota_wallet::account_manager::AccountManager;
  /// use iota_wallet::client::ClientOptionsBuilder;
  ///
  /// // gets 10 received transactions, skipping the first 5 most recent transactions.
  /// let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
  ///  .expect("invalid node URL")
  ///  .build();
  /// let mut manager = AccountManager::new();
  /// let mut account = manager.create_account(client_options)
  ///   .initialise()
  ///   .expect("failed to add account");
  /// account.list_transactions(10, 5, Some(TransactionType::Received));
  /// ```
  pub fn list_transactions(
    &self,
    count: u64,
    from: u64,
    transaction_type: Option<TransactionType>,
  ) -> Vec<&Transaction> {
    self
      .transactions
      .iter()
      .filter(|tx| {
        if let Some(tx_type) = transaction_type.clone() {
          match tx_type {
            TransactionType::Received => self.addresses.contains(tx.address()),
            TransactionType::Sent => !self.addresses.contains(tx.address()),
            TransactionType::Failed => !tx.broadcasted(),
            TransactionType::Unconfirmed => !tx.confirmed(),
            TransactionType::Value => tx.value().without_denomination() > 0,
          }
        } else {
          true
        }
      })
      .collect()
  }

  /// Gets the addresses linked to this account.
  ///
  /// * `unspent` - Whether it should get only unspent addresses or not.
  pub fn list_addresses(&mut self, unspent: bool) -> Vec<&Address> {
    self
      .addresses
      .iter()
      .filter(|address| crate::address::is_unspent(&self, address.address()) == unspent)
      .collect()
  }

  /// Gets a new unused address and links it to this account.
  pub async fn generate_address(&mut self) -> crate::Result<Address> {
    let address = crate::address::get_new_address(&self).await?;
    self.addresses.push(address.clone());
    crate::storage::get_adapter()?.set(self.id.to_string().into(), serde_json::to_string(self)?)?;
    Ok(address)
  }

  pub(crate) fn append_transactions(&mut self, transactions: Vec<Transaction>) {
    self.transactions.extend(transactions.iter().cloned());
  }

  /// Gets a transaction with the given hash associated with this account.
  pub fn get_transaction(&self, hash: &Hash) -> Option<&Transaction> {
    self.transactions.iter().find(|tx| tx.hash() == hash)
  }
}

/// Data returned from the account initialisation.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct InitialisedAccount<'a> {
  /// The account identifier.
  id: &'a str,
  /// The account alias.
  alias: &'a str,
  /// Seed address history.
  addresses: Vec<Address>,
  /// Seed transaction history.
  transactions: Vec<Transaction>,
  /// Account creation time.
  created_at: DateTime<Utc>,
  /// Time when the account was last synced with the tangle.
  last_synced_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
  use crate::account_manager::AccountManager;
  use crate::client::ClientOptionsBuilder;

  #[test]
  fn set_alias() {
    let manager = AccountManager::new();
    let id = "test";
    let updated_alias = "updated alias";
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
      .expect("invalid node URL")
      .build();

    let mut account = manager
      .create_account(client_options)
      .alias(id)
      .id(id)
      .mnemonic(id)
      .initialise()
      .expect("failed to add account");

    account
      .set_alias(updated_alias)
      .expect("failed to update alias");
    let account_in_storage = manager
      .get_account(id.to_string().into())
      .expect("failed to get account from storage");
    assert_eq!(
      account_in_storage.alias().to_string(),
      updated_alias.to_string()
    );
  }
}

use crate::address::Address;
use crate::client::ClientOptions;
use crate::storage::TransactionType;
use crate::transaction::Transaction;

use bee_crypto::ternary::Kerl;
use bee_signing::ternary::TernarySeed;
use chrono::prelude::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

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
pub struct AccountInitialiser<'a> {
  mnemonic: Option<&'a str>,
  id: Option<&'a str>,
  alias: Option<&'a str>,
  created_at: Option<DateTime<Utc>>,
  transactions: Vec<Transaction>,
  addresses: Vec<Address>,
  client_options: ClientOptions,
}

impl<'a> AccountInitialiser<'a> {
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
  pub fn mnemonic(mut self, mnemonic: &'a str) -> Self {
    self.mnemonic = Some(mnemonic);
    self
  }

  /// SHA-256 hash of the first address on the seed.
  /// Required for referencing a seed in stronghold.
  /// The id should be provided by stronghold.
  pub fn id(mut self, id: &'a str) -> Self {
    self.id = Some(id);
    self
  }

  /// Defines the account alias. If not defined, we'll generate one.
  pub fn alias(mut self, alias: &'a str) -> Self {
    self.alias = Some(alias);
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
  pub fn initialise(self) -> crate::Result<Account<'a>> {
    let alias = self.alias.unwrap_or_else(|| "");
    let id = self.id.unwrap_or(alias);

    let account = Account {
      id,
      alias,
      created_at: self.created_at.unwrap_or_else(chrono::Utc::now),
      transactions: self.transactions,
      addresses: self.addresses,
      client_options: self.client_options,
    };
    let adapter = crate::storage::get_adapter()?;
    adapter.set(id.to_string().into(), serde_json::to_string(&account)?)?;
    Ok(account)
  }
}

/// Account definition.
#[derive(Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Account<'a> {
  /// The account identifier.
  id: &'a str,
  /// The account alias.
  alias: &'a str,
  /// Time of account creation.
  created_at: DateTime<Utc>,
  /// Transactions associated with the seed.
  /// The account can be initialised with locally stored transactions.
  transactions: Vec<Transaction>,
  /// Address history associated with the seed.
  /// The account can be initialised with locally stored address history.
  addresses: Vec<Address>,
  /// The client options.
  client_options: ClientOptions,
}

impl<'a> Account<'a> {
  pub(crate) fn latest_address(&self) -> &Address {
    &self.addresses.iter().max_by_key(|a| a.key_index()).unwrap()
  }

  pub(crate) fn seed(&self) -> &TernarySeed<Kerl> {
    unimplemented!()
  }

  /// Gets the account's total balance.
  /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
  pub fn total_balance(&mut self) -> crate::Result<u64> {
    let id = self.alias;
    crate::storage::total_balance(id)
  }

  /// Gets the account's available balance.
  /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
  ///
  /// The available balance is the balance users are allowed to spend.
  /// For example, if a user with 50i total account balance has made a transaction spending 20i,
  /// the available balance should be (50i-30i) = 20i.
  pub fn available_balance(&mut self) -> crate::Result<u64> {
    let id = self.alias;
    crate::storage::available_balance(id)
  }

  /// Updates the account alias.
  pub fn set_alias(&mut self, alias: &str) -> crate::Result<()> {
    let id = self.alias;
    crate::storage::set_alias(id, alias)
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
  /// use iota_wallet::storage::TransactionType;
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
    &mut self,
    count: u64,
    from: u64,
    transaction_type: Option<TransactionType>,
  ) -> Vec<&Transaction> {
    let id = self.alias;
    self
      .transactions
      .iter()
      .filter(|tx| {
        if let Some(tx_type) = transaction_type.clone() {
          true
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
  pub fn generate_address(&mut self) -> crate::Result<Address> {
    let id = self.alias;
    let address = crate::address::get_new_address(&self)?;
    crate::storage::save_address(id, &address)
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

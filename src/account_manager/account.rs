use crate::address::Address;
use crate::storage::TransactionType;
use crate::transaction::Transaction;

use chrono::prelude::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

/// Network type.
#[derive(Serialize, Deserialize)]
pub enum Network {
  /// IOTA's main network.
  Mainnet,
  /// IOTA's dev network.
  Devnet,
  /// IOTA's community network.
  Comnet,
}

/// Account initialiser.
#[derive(Default)]
pub struct AccountInitialiser<'a> {
  mnemonic: Option<&'a str>,
  id: Option<&'a str>,
  alias: Option<&'a str>,
  nodes: Option<Vec<&'a str>>,
  quorum_size: Option<u64>,
  quorum_threshold: Option<u64>,
  network: Option<Network>,
  provider: Option<&'a str>,
  created_at: Option<DateTime<Utc>>,
  transactions: Vec<Transaction<'a>>,
  addresses: Vec<Address>,
}

impl<'a> AccountInitialiser<'a> {
  /// Initialises the account builder.
  pub(crate) fn new() -> Self {
    Default::default()
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

  /// Defines the list of nodes to connect to.
  pub fn nodes(mut self, nodes: Vec<&'a str>) -> Self {
    self.nodes = Some(nodes);
    self
  }

  /// Defines the quorum size.
  /// If multiple nodes are defined, the quorum size determines
  /// the number of nodes to query to check for quorum.
  pub fn quorum_size(mut self, quorum_size: u64) -> Self {
    self.quorum_size = Some(quorum_size);
    self
  }

  /// Defines the minimum number of nodes from the quorum pool
  /// that need to agree for considering the result as true.
  pub fn quorum_threshold(mut self, quorum_threshold: u64) -> Self {
    self.quorum_threshold = Some(quorum_threshold);
    self
  }

  /// Defines the IOTA public network to use.
  pub fn network(mut self, network: Network) -> Self {
    self.network = Some(network);
    self
  }

  /// Node URL.
  pub fn provider(mut self, provider: &'a str) -> Self {
    self.provider = Some(provider);
    self
  }

  /// Time of account creation.
  pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
    self.created_at = Some(created_at);
    self
  }

  /// Transactions associated with the seed.
  /// The account can be initialised with locally stored transactions.
  pub fn transactions(mut self, transactions: Vec<Transaction<'a>>) -> Self {
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

    let account = Account {
      id: self.id.unwrap_or(alias),
      alias,
      nodes: self
        .nodes
        .ok_or_else(|| anyhow::anyhow!("the `nodes` array is required"))?,
      quorum_size: self.quorum_size,
      quorum_threshold: self.quorum_threshold,
      network: self.network,
      provider: self.provider,
      created_at: self.created_at.unwrap_or_else(chrono::Utc::now),
      transactions: self.transactions,
      addresses: self.addresses,
    };
    Ok(account)
  }
}

/// Account definition.
#[derive(Getters, Serialize, Deserialize)]
pub struct Account<'a> {
  /// The account identifier.
  #[getset(get = "pub")]
  id: &'a str,
  /// The account alias.
  #[getset(get = "pub")]
  alias: &'a str,
  /// The list of nodes to connect to.
  #[getset(get = "pub")]
  nodes: Vec<&'a str>,
  /// The quorum size.
  /// If multiple nodes are defined, the quorum size determines
  /// the number of nodes to query to check for quorum.
  #[getset(get = "pub")]
  quorum_size: Option<u64>,
  /// The minimum number of nodes from the quorum pool
  /// that need to agree for considering the result as true.
  #[getset(get = "pub")]
  quorum_threshold: Option<u64>,
  /// The IOTA public network to use.
  #[getset(get = "pub")]
  network: Option<Network>,
  /// Node URL.
  #[getset(get = "pub")]
  provider: Option<&'a str>,
  /// Time of account creation.
  #[getset(get = "pub")]
  created_at: DateTime<Utc>,
  /// Transactions associated with the seed.
  /// The account can be initialised with locally stored transactions.
  #[getset(get = "pub")]
  transactions: Vec<Transaction<'a>>,
  /// Address history associated with the seed.
  /// The account can be initialised with locally stored address history.
  #[getset(get = "pub")]
  addresses: Vec<Address>,
}

impl<'a> Account<'a> {
  pub(crate) fn new(account_id: &'a str) -> Self {
    Self {
      id: account_id,
      alias: account_id,
      nodes: vec![],
      quorum_size: None,
      quorum_threshold: None,
      network: None,
      provider: None,
      created_at: Utc::now(),
      transactions: vec![],
      addresses: vec![],
    }
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

  /// Gets a list of transactions on the given account.
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
  ///
  /// // gets 10 received transactions, skipping the first 5 most recent transactions.
  /// let mut manager = AccountManager::new();
  /// let mut account = manager.create_account()
  ///   .nodes(vec!["https://nodes.devnet.iota.org:443"])
  ///   .initialise()
  ///   .expect("failed to add account");
  /// account.list_transactions(10, 5, Some(TransactionType::Received));
  /// ```
  pub fn list_transactions(
    &mut self,
    count: u64,
    from: u64,
    transaction_type: Option<TransactionType>,
  ) -> crate::Result<Vec<Transaction<'a>>> {
    let id = self.alias;
    crate::storage::list_transactions(id, count, from, transaction_type)
  }

  /// Gets the addresses linked to the given account.
  ///
  /// * `unspent` - Whether it should get only unspent addresses or not.
  pub fn list_addresses(&mut self, unspent: bool) -> crate::Result<Vec<Address>> {
    let id = self.alias;
    crate::storage::list_addresses(id, unspent)
  }

  /// Gets a new unused address and links it to the given account.
  pub fn generate_address(&mut self) -> crate::Result<Address> {
    let id = self.alias;
    crate::storage::generate_address(id)
  }
}

/// Data returned from the account initialisation.
#[derive(Getters)]
pub struct InitialisedAccount<'a> {
  /// The account identifier.
  #[getset(get = "pub")]
  id: &'a str,
  /// The account alias.
  #[getset(get = "pub")]
  alias: &'a str,
  /// Seed address history.
  #[getset(get = "pub")]
  addresses: Vec<Address>,
  /// Seed transaction history.
  #[getset(get = "pub")]
  transactions: Vec<Transaction<'a>>,
  /// Account creation time.
  #[getset(get = "pub")]
  created_at: DateTime<Utc>,
  /// Time when the account was last synced with the tangle.
  #[getset(get = "pub")]
  last_synced_at: DateTime<Utc>,
}

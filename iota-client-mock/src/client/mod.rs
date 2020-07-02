use bee_crypto::ternary::Hash;
use bee_transaction::bundled::Address;

mod send;
use send::SendBuilder;

mod transaction;
use transaction::{FindTransactionsBuilder, GetTransactionBuilder};

mod get_balance;
use get_balance::{GetBalanceBuilder, GetBalanceForAddressBuilder};

mod generate_address;
use generate_address::GenerateAddressBuilder;

pub enum Converter {
  UTF8,
  Bytes,
}

pub enum Network {
  Mainnet,
  Devnet,
  Comnet,
}

#[derive(Default)]
pub struct SingleNodeClientBuilder<'a> {
  node: &'a str,
  mwm: Option<u64>,
  checksum_required: bool,
}

impl<'a> SingleNodeClientBuilder<'a> {
  fn new(node: &'a str) -> Self {
    Self {
      node,
      checksum_required: true,
      ..Default::default()
    }
  }

  pub fn mwm(mut self, mwm: u64) -> Self {
    self.mwm = Some(mwm);
    self
  }

  pub fn checksum_required(mut self, checksum_required: bool) -> Self {
    self.checksum_required = checksum_required;
    self
  }

  pub fn build(self) -> Client<'a> {
    Client {
      node: Some(self.node),
      nodes: None,
      node_pool_urls: None,
      network: None,
      mwm: self.mwm,
      quorum_size: None,
      quorum_threshold: 0.0,
      checksum_required: self.checksum_required,
    }
  }
}

pub struct MultiNodeClientBuilder<'a> {
  nodes: Option<&'a [&'a str]>,
  node_pool_urls: Option<&'a [&'a str]>,
  network: Option<Network>,
  mwm: Option<u64>,
  quorum_size: Option<u64>,
  quorum_threshold: f32,
  checksum_required: bool,
  // state_adapter:
}

impl Default for MultiNodeClientBuilder<'_> {
  fn default() -> Self {
    Self {
      nodes: None,
      node_pool_urls: None,
      network: None,
      mwm: None,
      quorum_size: None,
      quorum_threshold: 0.5,
      checksum_required: true,
    }
  }
}

impl<'a> MultiNodeClientBuilder<'a> {
  fn with_nodes(nodes: &'a [&'a str]) -> Self {
    Self {
      nodes: Some(nodes),
      ..Default::default()
    }
  }

  fn with_node_pool(node_pool_urls: &'a [&'a str]) -> Self {
    Self {
      node_pool_urls: Some(node_pool_urls),
      ..Default::default()
    }
  }

  fn with_network(network: Network) -> Self {
    Self {
      network: Some(network),
      ..Default::default()
    }
  }

  pub fn mwm(mut self, mwm: u64) -> Self {
    self.mwm = Some(mwm);
    self
  }

  pub fn quorum_size(mut self, quorum_size: u64) -> Self {
    self.quorum_size = Some(quorum_size);
    self
  }

  pub fn quorum_threshold(mut self, quorum_threshold: f32) -> Self {
    self.quorum_threshold = quorum_threshold;
    self
  }

  pub fn checksum_required(mut self, checksum_required: bool) -> Self {
    self.checksum_required = checksum_required;
    self
  }

  pub fn build(self) -> Client<'a> {
    Client {
      node: None,
      nodes: self.nodes,
      node_pool_urls: self.node_pool_urls,
      network: self.network,
      mwm: self.mwm,
      quorum_size: self.quorum_size,
      quorum_threshold: self.quorum_threshold,
      checksum_required: self.checksum_required,
    }
  }
}

#[derive(Default)]
pub struct ClientBuilder<'a> {
  node: Option<&'a str>,
  nodes: Option<&'a [&'a str]>,
  node_pool_urls: Option<&'a [&'a str]>,
  network: Option<Network>,
}

impl<'a> ClientBuilder<'a> {
  pub fn new() -> Self {
    Default::default()
  }

  /// Client connected to a single node.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::new()
  ///   .node("https://nodes.devnet.iota.org:443")
  ///   .build();
  /// ```
  pub fn node(self, node: &'a str) -> SingleNodeClientBuilder<'a> {
    SingleNodeClientBuilder::new(node)
  }

  /// Client connected to a list of nodes.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::new()
  ///   .nodes(&["https://nodes.devnet.iota.org:443", "https://nodes.comnet.thetangle.org/"])
  ///   .build();
  /// ```
  pub fn nodes(self, nodes: &'a [&'a str]) -> MultiNodeClientBuilder<'a> {
    MultiNodeClientBuilder::with_nodes(nodes)
  }

  /// Client connected to the response of a pool.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::new()
  ///   .node_pool_urls(&["https://nodes.iota.works/api/ssl/live"])
  ///   .build();
  /// ```
  pub fn node_pool_urls(self, node_pool_urls: &'a [&'a str]) -> MultiNodeClientBuilder<'a> {
    MultiNodeClientBuilder::with_node_pool(node_pool_urls)
  }

  /// Client connected to the default Network pool.
  ///
  /// # Examples
  /// ```
  /// use iota_client::{ClientBuilder, Network};
  /// let client = ClientBuilder::new()
  ///   .network(Network::Devnet)
  ///   .build();
  /// ```
  pub fn network(self, network: Network) -> MultiNodeClientBuilder<'a> {
    MultiNodeClientBuilder::with_network(network)
  }
}

#[derive(Default)]
pub struct Client<'a> {
  node: Option<&'a str>,
  nodes: Option<&'a [&'a str]>,
  node_pool_urls: Option<&'a [&'a str]>,
  network: Option<Network>,
  mwm: Option<u64>,
  quorum_size: Option<u64>,
  quorum_threshold: f32,
  checksum_required: bool,
}

impl<'a> Client<'a> {
  pub fn send(address: Address) -> SendBuilder<'a> {
    SendBuilder::new(address)
  }

  pub fn transaction(transaction_hash: Hash) -> GetTransactionBuilder {
    GetTransactionBuilder::new(transaction_hash)
  }

  pub fn transactions() -> FindTransactionsBuilder {
    FindTransactionsBuilder::new()
  }

  pub fn generate_address() -> GenerateAddressBuilder<'a> {
    GenerateAddressBuilder::new()
  }

  pub fn balance() -> GetBalanceBuilder<'a> {
    GetBalanceBuilder::new()
  }

  pub fn balance_for_address(address: Address) -> GetBalanceForAddressBuilder {
    GetBalanceForAddressBuilder::new(address)
  }
}

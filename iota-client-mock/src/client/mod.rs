use bee_crypto::ternary::Hash;
use bee_transaction::bundled::Address;
use serde::{Deserialize, Serialize};
use url::Url;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Network {
  Mainnet,
  Devnet,
  Comnet,
}

pub struct SingleNodeClientBuilder {
  node: Url,
  mwm: Option<u64>,
  checksum_required: bool,
}

impl SingleNodeClientBuilder {
  fn new(node: &str) -> crate::Result<Self> {
    let node_url = Url::parse(node)?;
    let builder = Self {
      node: node_url,
      mwm: None,
      checksum_required: true,
    };
    Ok(builder)
  }

  pub fn mwm(mut self, mwm: u64) -> Self {
    self.mwm = Some(mwm);
    self
  }

  pub fn checksum_required(mut self, checksum_required: bool) -> Self {
    self.checksum_required = checksum_required;
    self
  }

  pub fn build(self) -> Client {
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

pub struct MultiNodeClientBuilder {
  nodes: Option<Vec<Url>>,
  node_pool_urls: Option<Vec<Url>>,
  network: Option<Network>,
  mwm: Option<u64>,
  quorum_size: Option<u64>,
  quorum_threshold: f32,
  checksum_required: bool,
  // state_adapter:
}

fn convert_urls(urls: &[&str]) -> crate::Result<Vec<Url>> {
  let mut err = None;
  let urls: Vec<Option<Url>> = urls
    .iter()
    .map(|node| {
      Url::parse(node).map(|url| Some(url)).unwrap_or_else(|e| {
        err = Some(e);
        None
      })
    })
    .collect();

  if let Some(err) = err {
    Err(err.into())
  } else {
    let urls = urls.iter().map(|url| url.clone().unwrap()).collect();
    Ok(urls)
  }
}

impl Default for MultiNodeClientBuilder {
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

impl MultiNodeClientBuilder {
  fn with_nodes(nodes: &[&str]) -> crate::Result<Self> {
    let nodes_urls = convert_urls(nodes)?;
    let builder = Self {
      nodes: Some(nodes_urls),
      ..Default::default()
    };
    Ok(builder)
  }

  fn with_node_pool(node_pool_urls: &[&str]) -> crate::Result<Self> {
    let pool_urls = convert_urls(node_pool_urls)?;
    let builder = Self {
      node_pool_urls: Some(pool_urls),
      ..Default::default()
    };
    Ok(builder)
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

  pub fn build(self) -> Client {
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

pub struct ClientBuilder;

impl ClientBuilder {
  /// Client connected to a single node.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::node("https://nodes.devnet.iota.org:443")
  ///   .expect("invalid node URL")
  ///   .build();
  /// ```
  pub fn node(node: &str) -> crate::Result<SingleNodeClientBuilder> {
    SingleNodeClientBuilder::new(node)
  }

  /// Client connected to a list of nodes.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::nodes(&["https://nodes.devnet.iota.org:443", "https://nodes.comnet.thetangle.org/"])
  ///   .expect("invalid nodes URLs")
  ///   .build();
  /// ```
  pub fn nodes(nodes: &[&str]) -> crate::Result<MultiNodeClientBuilder> {
    MultiNodeClientBuilder::with_nodes(nodes)
  }

  /// Client connected to the response of a pool.
  ///
  /// # Examples
  /// ```
  /// use iota_client::ClientBuilder;
  /// let client = ClientBuilder::node_pool_urls(&["https://nodes.iota.works/api/ssl/live"])
  ///   .expect("invalid pool URLs")
  ///   .build();
  /// ```
  pub fn node_pool_urls(node_pool_urls: &[&str]) -> crate::Result<MultiNodeClientBuilder> {
    MultiNodeClientBuilder::with_node_pool(node_pool_urls)
  }

  /// Client connected to the default Network pool.
  ///
  /// # Examples
  /// ```
  /// use iota_client::{ClientBuilder, Network};
  /// let client = ClientBuilder::network(Network::Devnet)
  ///   .build();
  /// ```
  pub fn network(network: Network) -> MultiNodeClientBuilder {
    MultiNodeClientBuilder::with_network(network)
  }
}

#[derive(Default)]
pub struct Client {
  node: Option<Url>,
  nodes: Option<Vec<Url>>,
  node_pool_urls: Option<Vec<Url>>,
  network: Option<Network>,
  mwm: Option<u64>,
  quorum_size: Option<u64>,
  quorum_threshold: f32,
  checksum_required: bool,
}

impl Client {
  pub fn send<'a>(&self, address: Address) -> SendBuilder<'a> {
    SendBuilder::new(address)
  }

  pub fn transaction(&self, transaction_hash: Hash) -> GetTransactionBuilder {
    GetTransactionBuilder::new(transaction_hash)
  }

  pub fn transactions(&self) -> FindTransactionsBuilder {
    FindTransactionsBuilder::new()
  }

  pub fn generate_address<'a>(&self) -> GenerateAddressBuilder<'a> {
    GenerateAddressBuilder::new()
  }

  pub fn balance<'a>(&self) -> GetBalanceBuilder<'a> {
    GetBalanceBuilder::new()
  }

  pub fn balance_for_address<'a>(&self, address: &'a Address) -> GetBalanceForAddressBuilder<'a> {
    GetBalanceForAddressBuilder::new(address)
  }
}

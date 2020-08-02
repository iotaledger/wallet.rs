pub use iota::client::builder::Network;
use iota::client::{Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use url::Url;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ClientInstanceMap = Arc<Mutex<HashMap<ClientOptions, Client>>>;

/// Gets the balance change listeners array.
fn instances() -> &'static ClientInstanceMap {
  static LISTENERS: Lazy<ClientInstanceMap> = Lazy::new(Default::default);
  &LISTENERS
}

pub(crate) fn with_client<T, F: FnOnce(&Client) -> T>(options: &ClientOptions, cb: F) -> T {
  let mut map = instances()
    .lock()
    .expect("failed to lock client instances: get_client()");

  if !map.contains_key(&options) {
    let client = ClientBuilder::new()
      .node("http://127.0.0.1:8080")
      .expect("failed to initialise ClientBuilder")
      .build()
      .expect("failed to initialise ClientBuilder");

    map.insert(options.clone(), client);
  }

  let client = map.get(&options).expect("client not initialised");
  cb(client)
}

/// The options builder for a client connected to a single node.
pub struct SingleNodeClientOptionsBuilder {
  node: Url,
  mwm: Option<u64>,
  checksum_required: bool,
}

impl SingleNodeClientOptionsBuilder {
  fn new(node: &str) -> crate::Result<Self> {
    let node_url = Url::parse(node)?;
    let builder = Self {
      node: node_url,
      mwm: None,
      checksum_required: true,
    };
    Ok(builder)
  }

  /// Sets the mwm.
  pub fn mwm(mut self, mwm: u64) -> Self {
    self.mwm = Some(mwm);
    self
  }

  /// Whether the checksum is required or not.
  pub fn checksum_required(mut self, checksum_required: bool) -> Self {
    self.checksum_required = checksum_required;
    self
  }

  /// Builds the options.
  pub fn build(self) -> ClientOptions {
    ClientOptions {
      node: Some(self.node),
      nodes: None,
      node_pool_urls: None,
      network: None,
      mwm: self.mwm,
      quorum_size: None,
      quorum_threshold: 0,
      checksum_required: self.checksum_required,
    }
  }
}

/// The options builder for a client connected to multiple nodes.
pub struct MultiNodeClientOptionsBuilder {
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

impl Default for MultiNodeClientOptionsBuilder {
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

impl MultiNodeClientOptionsBuilder {
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

  /// Sets the mwm.
  pub fn mwm(mut self, mwm: u64) -> Self {
    self.mwm = Some(mwm);
    self
  }

  /// Sets the quorum size.
  pub fn quorum_size(mut self, quorum_size: u64) -> Self {
    self.quorum_size = Some(quorum_size);
    self
  }

  /// Sets the quorum threshold.
  pub fn quorum_threshold(mut self, quorum_threshold: f32) -> Self {
    self.quorum_threshold = quorum_threshold;
    self
  }

  /// Whether the address checksum is required or not.
  pub fn checksum_required(mut self, checksum_required: bool) -> Self {
    self.checksum_required = checksum_required;
    self
  }

  /// Builds the options.
  pub fn build(self) -> ClientOptions {
    ClientOptions {
      node: None,
      nodes: self.nodes,
      node_pool_urls: self.node_pool_urls,
      network: self.network,
      mwm: self.mwm,
      quorum_size: self.quorum_size,
      quorum_threshold: (self.quorum_threshold * 100.0) as u32,
      checksum_required: self.checksum_required,
    }
  }
}

/// The ClientOptions builder.
pub struct ClientOptionsBuilder;

impl ClientOptionsBuilder {
  /// Client connected to a single node.
  ///
  /// # Examples
  /// ```
  /// use iota_wallet::client::ClientOptionsBuilder;
  /// let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
  ///   .expect("invalid node URL")
  ///   .build();
  /// ```
  pub fn node(node: &str) -> crate::Result<SingleNodeClientOptionsBuilder> {
    SingleNodeClientOptionsBuilder::new(node)
  }

  /// ClientOptions connected to a list of nodes.
  ///
  /// # Examples
  /// ```
  /// use iota_wallet::client::ClientOptionsBuilder;
  /// let client_options = ClientOptionsBuilder::nodes(&["https://nodes.devnet.iota.org:443", "https://nodes.comnet.thetangle.org/"])
  ///   .expect("invalid nodes URLs")
  ///   .build();
  /// ```
  pub fn nodes(nodes: &[&str]) -> crate::Result<MultiNodeClientOptionsBuilder> {
    MultiNodeClientOptionsBuilder::with_nodes(nodes)
  }

  /// ClientOptions connected to the response of a pool.
  ///
  /// # Examples
  /// ```
  /// use iota_wallet::client::ClientOptionsBuilder;
  /// let client_options = ClientOptionsBuilder::node_pool_urls(&["https://nodes.iota.works/api/ssl/live"])
  ///   .expect("invalid pool URLs")
  ///   .build();
  /// ```
  pub fn node_pool_urls(node_pool_urls: &[&str]) -> crate::Result<MultiNodeClientOptionsBuilder> {
    MultiNodeClientOptionsBuilder::with_node_pool(node_pool_urls)
  }

  /// ClientOptions connected to the default Network pool.
  ///
  /// # Examples
  /// ```
  /// use iota_wallet::client::{ClientOptionsBuilder, Network};
  /// let client_options = ClientOptionsBuilder::network(Network::Devnet)
  ///   .build();
  /// ```
  pub fn network(network: Network) -> MultiNodeClientOptionsBuilder {
    MultiNodeClientOptionsBuilder::with_network(network)
  }
}

/// The client options type.
#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClientOptions {
  node: Option<Url>,
  nodes: Option<Vec<Url>>,
  node_pool_urls: Option<Vec<Url>>,
  network: Option<Network>,
  mwm: Option<u64>,
  quorum_size: Option<u64>,
  quorum_threshold: u32,
  checksum_required: bool,
}

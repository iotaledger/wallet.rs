use getset::Getters;
pub use iota::client::builder::Network;
use iota::client::{Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use url::Url;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

type ClientInstanceMap = Arc<Mutex<HashMap<ClientOptions, Arc<RwLock<Client>>>>>;

/// Gets the client instances map.
fn instances() -> &'static ClientInstanceMap {
    static INSTANCES: Lazy<ClientInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) fn get_client(options: &ClientOptions) -> Arc<RwLock<Client>> {
    let mut map = instances()
        .lock()
        .expect("failed to lock client instances: get_client()");

    if !map.contains_key(&options) {
        let mut client_builder = ClientBuilder::new().quorum_threshold(*options.quorum_threshold());

        // we validate the URL beforehand so it's safe to unwrap here
        if let Some(node) = options.node() {
            client_builder = client_builder.node(node.as_str()).unwrap();
        } else if let Some(nodes) = options.nodes() {
            client_builder = client_builder
                .nodes(&nodes.iter().map(|url| url.as_str()).collect::<Vec<&str>>()[..])
                .unwrap();
        }

        if let Some(network) = options.network() {
            client_builder = client_builder.network(network.clone());
        }

        if let Some(quorum_size) = options.quorum_size() {
            client_builder = client_builder.quorum_size(*quorum_size);
        }

        let client = client_builder
            .build()
            .expect("failed to initialise ClientBuilder");

        map.insert(options.clone(), Arc::new(RwLock::new(client)));
    }

    let client = map.get(&options).expect("client not initialised");
    client.clone()
}

/// The options builder for a client connected to a single node.
pub struct SingleNodeClientOptionsBuilder {
    node: Url,
}

impl SingleNodeClientOptionsBuilder {
    fn new(node: &str) -> crate::Result<Self> {
        let node_url = Url::parse(node)?;
        let builder = Self { node: node_url };
        Ok(builder)
    }

    /// Builds the options.
    pub fn build(self) -> ClientOptions {
        ClientOptions {
            node: Some(self.node),
            nodes: None,
            network: None,
            quorum_size: None,
            quorum_threshold: 0,
        }
    }
}

/// The options builder for a client connected to multiple nodes.
pub struct MultiNodeClientOptionsBuilder {
    nodes: Option<Vec<Url>>,
    network: Option<Network>,
    quorum_size: Option<u8>,
    quorum_threshold: f32,
    // state_adapter:
}

fn convert_urls(urls: &[&str]) -> crate::Result<Vec<Url>> {
    let mut err = None;
    let urls: Vec<Option<Url>> = urls
        .iter()
        .map(|node| {
            Url::parse(node).map(Some).unwrap_or_else(|e| {
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
            network: None,
            quorum_size: None,
            quorum_threshold: 0.5,
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

    fn with_network(network: Network) -> Self {
        Self {
            network: Some(network),
            ..Default::default()
        }
    }

    /// Sets the nodes.
    pub fn nodes(mut self, nodes: &[&str]) -> crate::Result<Self> {
        let nodes_urls = convert_urls(nodes)?;
        self.nodes = Some(nodes_urls);
        Ok(self)
    }

    /// Sets the IOTA network the nodes belong to.
    pub fn network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets the quorum size.
    pub fn quorum_size(mut self, quorum_size: u8) -> Self {
        self.quorum_size = Some(quorum_size);
        self
    }

    /// Sets the quorum threshold.
    pub fn quorum_threshold(mut self, quorum_threshold: f32) -> Self {
        self.quorum_threshold = quorum_threshold;
        self
    }

    /// Builds the options.
    pub fn build(self) -> crate::Result<ClientOptions> {
        let node_len = match &self.nodes {
            Some(nodes) => nodes.len(),
            None => 0,
        };
        if node_len == 0 {
            return Err(crate::WalletError::EmptyNodeList);
        }
        let options = ClientOptions {
            node: None,
            nodes: self.nodes,
            network: self.network,
            quorum_size: self.quorum_size,
            quorum_threshold: (self.quorum_threshold * 100.0) as u8,
        };
        Ok(options)
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
    /// let client_options = ClientOptionsBuilder::node("https://tangle.iotaqubic.us:14267")
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
    /// let client_options = ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267", "https://gewirr.com:14267/"])
    ///   .expect("invalid nodes URLs")
    ///   .build();
    /// ```
    pub fn nodes(nodes: &[&str]) -> crate::Result<MultiNodeClientOptionsBuilder> {
        MultiNodeClientOptionsBuilder::with_nodes(nodes)
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
#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub(crate)")]
pub struct ClientOptions {
    node: Option<Url>,
    nodes: Option<Vec<Url>>,
    network: Option<Network>,
    #[serde(rename = "quorumSize")]
    quorum_size: Option<u8>,
    #[serde(rename = "quorumThreshold", default)]
    quorum_threshold: u8,
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
pub use iota::client::builder::Network;
use iota::client::{BrokerOptions, Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
        let mut client_builder = ClientBuilder::new()
            .with_mqtt_broker_options(BrokerOptions::new().automatic_disconnect(false))
            .with_local_pow(*options.local_pow());

        // we validate the URL beforehand so it's safe to unwrap here
        if let Some(node) = options.node() {
            client_builder = client_builder.with_node(node.as_str()).unwrap();
        } else if let Some(nodes) = options.nodes() {
            client_builder = client_builder
                .with_nodes(&nodes.iter().map(|url| url.as_str()).collect::<Vec<&str>>()[..])
                .unwrap();
        }

        if let Some(network) = options.network() {
            client_builder = client_builder.with_network(network.clone());
        }

        let client = client_builder.finish().expect("failed to initialise ClientBuilder");

        map.insert(options.clone(), Arc::new(RwLock::new(client)));
    }

    let client = map.get(&options).expect("client not initialised");
    client.clone()
}

/// The options builder for a client connected to a single node.
pub struct SingleNodeClientOptionsBuilder {
    node: Url,
    local_pow: bool,
}

impl SingleNodeClientOptionsBuilder {
    fn new(node: &str) -> crate::Result<Self> {
        let node_url = Url::parse(node)?;
        let builder = Self {
            node: node_url,
            local_pow: default_local_pow(),
        };
        Ok(builder)
    }

    /// Sets the pow option.
    pub fn local_pow(mut self, local_pow: bool) -> Self {
        self.local_pow = local_pow;
        self
    }

    /// Builds the options.
    pub fn build(self) -> ClientOptions {
        ClientOptions {
            node: Some(self.node),
            nodes: None,
            network: None,
            quorum_size: None,
            quorum_threshold: 0,
            local_pow: self.local_pow,
        }
    }
}

/// The options builder for a client connected to multiple nodes.
pub struct MultiNodeClientOptionsBuilder {
    nodes: Option<Vec<Url>>,
    network: Option<Network>,
    quorum_size: Option<u8>,
    quorum_threshold: f32,
    local_pow: bool,
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
            local_pow: default_local_pow(),
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

    /// Sets the pow option.
    pub fn local_pow(mut self, local_pow: bool) -> Self {
        self.local_pow = local_pow;
        self
    }

    /// Builds the options.
    pub fn build(self) -> crate::Result<ClientOptions> {
        let options = ClientOptions {
            node: None,
            nodes: self.nodes,
            network: self.network,
            quorum_size: self.quorum_size,
            quorum_threshold: (self.quorum_threshold * 100.0) as u8,
            local_pow: self.local_pow,
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
    ///     .expect("invalid node URL")
    ///     .build();
    /// ```
    pub fn node(node: &str) -> crate::Result<SingleNodeClientOptionsBuilder> {
        SingleNodeClientOptionsBuilder::new(node)
    }

    /// ClientOptions connected to a list of nodes.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options =
    ///     ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267", "https://gewirr.com:14267/"])
    ///         .expect("invalid nodes URLs")
    ///         .build();
    /// ```
    pub fn nodes(nodes: &[&str]) -> crate::Result<MultiNodeClientOptionsBuilder> {
        MultiNodeClientOptionsBuilder::with_nodes(nodes)
    }

    /// ClientOptions connected to the default Network pool.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::{ClientOptionsBuilder, Network};
    /// let client_options = ClientOptionsBuilder::network(Network::Testnet).build();
    /// ```
    pub fn network(network: Network) -> MultiNodeClientOptionsBuilder {
        MultiNodeClientOptionsBuilder::with_network(network)
    }
}

/// The client options type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub(crate)")]
pub struct ClientOptions {
    node: Option<Url>,
    nodes: Option<Vec<Url>>,
    network: Option<Network>,
    #[serde(rename = "quorumSize")]
    quorum_size: Option<u8>,
    #[serde(rename = "quorumThreshold", default)]
    quorum_threshold: u8,
    #[serde(rename = "localPow", default = "default_local_pow")]
    local_pow: bool,
}

fn default_local_pow() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{ClientOptionsBuilder, Network};

    #[test]
    fn single_node_valid_url() {
        let builder_res = ClientOptionsBuilder::node("https://tangle.iotaqubic.us:14267");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::node("some.invalid url");
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_valid_url() {
        let builder_res = ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"]);
        assert!(builder_res.is_ok());
    }

    #[test]
    fn multi_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::nodes(&["some.invalid url"]);
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_empty() {
        let builder_res = ClientOptionsBuilder::nodes(&[]).unwrap().build();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn network_node_empty() {
        let builder_res = ClientOptionsBuilder::network(Network::Testnet).build();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_constructor() {
        let node = "https://tangle.iotaqubic.us:14267";
        let node_url: url::Url = url::Url::parse(node).unwrap();
        let client = ClientOptionsBuilder::node(node).unwrap().build();
        assert_eq!(client.node(), &Some(node_url));
        assert!(client.nodes().is_none());
        assert!(client.network().is_none());
        assert!(client.quorum_size().is_none());
        assert_eq!(*client.quorum_threshold(), 0);
    }

    #[test]
    fn multi_node_constructor() {
        let nodes = ["https://tangle.iotaqubic.us:14267"];
        let quorum_size = 5;
        let quorum_threshold = 0.5;
        let client = ClientOptionsBuilder::nodes(&nodes)
            .unwrap()
            .quorum_size(quorum_size)
            .quorum_threshold(quorum_threshold)
            .build()
            .unwrap();
        assert!(client.node().is_none());
        assert_eq!(client.nodes(), &Some(super::convert_urls(&nodes).unwrap()));
        assert!(client.network().is_none());
        assert_eq!(*client.quorum_size(), Some(quorum_size));
        assert!((*client.quorum_threshold() as f32 / 100.0 - quorum_threshold).abs() < f32::EPSILON);
    }

    #[test]
    fn network_constructor() {
        let nodes = ["https://tangle.iotaqubic.us:14267"];
        let network = Network::Testnet;
        let quorum_size = 50;
        let quorum_threshold = 0.9;
        let client = ClientOptionsBuilder::network(network.clone())
            .quorum_size(quorum_size)
            .quorum_threshold(quorum_threshold)
            .nodes(&nodes)
            .unwrap()
            .build()
            .unwrap();
        assert!(client.node().is_none());
        assert_eq!(client.nodes(), &Some(super::convert_urls(&nodes).unwrap()));
        assert_eq!(*client.network(), Some(network));
        assert_eq!(*client.quorum_size(), Some(quorum_size));
        assert!((*client.quorum_threshold() as f32 / 100.0 - quorum_threshold).abs() < f32::EPSILON);
    }

    #[test]
    fn get_client() {
        let test_cases = vec![
            ClientOptionsBuilder::node("https://tangle.iotaqubic.us:14267")
                .unwrap()
                .build(),
            ClientOptionsBuilder::node("https://nodes.iota.cafe:443")
                .unwrap()
                .build(),
            ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"])
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://nodes.iota.cafe:443"])
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"])
                .unwrap()
                .quorum_size(55)
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .network(Network::Mainnet)
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://tangle.iotaqubic.us:14267"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .network(Network::Testnet)
                .build()
                .unwrap(),
            ClientOptionsBuilder::network(Network::Testnet)
                .nodes(&["https://node.deviceproof.org:443"])
                .unwrap()
                .build()
                .unwrap(),
        ];

        // assert that each different client_options create a new client instance
        for case in &test_cases {
            let len = super::instances().lock().unwrap().len();
            super::get_client(&case);
            assert_eq!(super::instances().lock().unwrap().len() - len, 1);
        }

        // assert that subsequent calls with options already initialized doesn't create new clients
        let len = super::instances().lock().unwrap().len();
        for case in &test_cases {
            super::get_client(&case);
            assert_eq!(super::instances().lock().unwrap().len(), len);
        }
    }
}

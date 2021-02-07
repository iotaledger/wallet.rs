// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota::client::{Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{Mutex, RwLock};
use url::Url;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

type ClientInstanceMap = Arc<Mutex<HashMap<ClientOptions, Arc<RwLock<Client>>>>>;

/// Gets the client instances map.
fn instances() -> &'static ClientInstanceMap {
    static INSTANCES: Lazy<ClientInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) async fn get_client(options: &ClientOptions) -> Arc<RwLock<Client>> {
    let mut map = instances().lock().await;

    if !map.contains_key(&options) {
        let mut client_builder = ClientBuilder::new()
            .with_mqtt_broker_options(
                options
                    .mqtt_broker_options()
                    .as_ref()
                    .map(|options| options.clone().into())
                    .unwrap_or_else(|| iota::BrokerOptions::new().automatic_disconnect(false)),
            )
            .with_local_pow(*options.local_pow())
            // we validate the URL beforehand so it's safe to unwrap here
            .with_nodes(&options.nodes().iter().map(|url| url.as_str()).collect::<Vec<&str>>()[..])
            .unwrap()
            .with_node_pool_urls(
                &options
                    .node_pool_urls()
                    .iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<String>>()[..],
            )
            .await
            // safe to unwrap since we're sure we have valid URLs
            .unwrap();

        if let Some(network) = options.network() {
            client_builder = client_builder.with_network(network);
        }

        if let Some(node) = options.node() {
            // safe to unwrap since we're sure we have valid URLs
            client_builder = client_builder.with_node(node.as_str()).unwrap();
        }

        if let Some(node_sync_interval) = options.node_sync_interval() {
            client_builder = client_builder.with_node_sync_interval(*node_sync_interval);
        }

        if !options.node_sync_enabled() {
            client_builder = client_builder.with_node_sync_disabled();
        }

        if let Some(request_timeout) = options.request_timeout() {
            client_builder = client_builder.with_request_timeout(*request_timeout);
        }

        for (api, timeout) in options.api_timeout() {
            client_builder = client_builder.with_api_timeout(api.clone().into(), *timeout);
        }

        let client = client_builder
            .finish()
            .await
            .expect("failed to initialise ClientBuilder");

        map.insert(options.clone(), Arc::new(RwLock::new(client)));
    }

    let client = map.get(&options).expect("client not initialised");
    client.clone()
}

/// The options builder for a client connected to multiple nodes.
pub struct ClientOptionsBuilder {
    nodes: Vec<Url>,
    node_pool_urls: Vec<Url>,
    network: Option<String>,
    mqtt_broker_options: Option<BrokerOptions>,
    local_pow: bool,
    node_sync_interval: Option<Duration>,
    node_sync_enabled: bool,
    request_timeout: Option<Duration>,
    api_timeout: HashMap<Api, Duration>,
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

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            node_pool_urls: Vec::new(),
            network: None,
            mqtt_broker_options: None,
            local_pow: default_local_pow(),
            node_sync_interval: None,
            node_sync_enabled: default_node_sync_enabled(),
            request_timeout: None,
            api_timeout: Default::default(),
        }
    }
}

impl ClientOptionsBuilder {
    /// Initialises a new instance of the builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// ClientOptions connected to a list of nodes.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new()
    ///     .with_nodes(&[
    ///         "https://api.lb-0.testnet.chrysalis2.com",
    ///         "https://api.hornet-1.testnet.chrysalis2.com/",
    ///     ])
    ///     .expect("invalid nodes URLs")
    ///     .build();
    /// ```
    pub fn with_nodes(mut self, nodes: &[&str]) -> crate::Result<Self> {
        let nodes_urls = convert_urls(nodes)?;
        self.nodes.extend(nodes_urls);
        Ok(self)
    }

    /// Adds a node to the node list.
    pub fn with_node(mut self, node: &str) -> crate::Result<Self> {
        self.nodes.push(Url::parse(node)?);
        Ok(self)
    }

    /// Get node list from the node_pool_urls
    pub fn with_node_pool_urls(mut self, node_pool_urls: &[&str]) -> crate::Result<Self> {
        let nodes_urls = convert_urls(node_pool_urls)?;
        self.node_pool_urls.extend(nodes_urls);
        Ok(self)
    }

    /// ClientOptions connected to the default Network pool.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new().with_network("testnet2").build();
    /// ```
    pub fn with_network<N: Into<String>>(mut self, network: N) -> Self {
        self.network = Some(network.into());
        self
    }

    /// Set the node sync interval
    pub fn with_node_sync_interval(mut self, node_sync_interval: Duration) -> Self {
        self.node_sync_interval = Some(node_sync_interval);
        self
    }

    /// Disables the node syncing process.
    /// Every node will be considered healthy and ready to use.
    pub fn with_node_sync_disabled(mut self) -> Self {
        self.node_sync_enabled = false;
        self
    }

    /// Sets the MQTT broker options.
    pub fn with_mqtt_mqtt_broker_options(mut self, options: BrokerOptions) -> Self {
        self.mqtt_broker_options = Some(options);
        self
    }

    /// Sets whether the PoW should be done locally or remotely.
    pub fn with_local_pow(mut self, local: bool) -> Self {
        self.local_pow = local;
        self
    }

    /// Sets the request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Sets the request timeout for a specific API usage.
    pub fn with_api_timeout(mut self, api: Api, timeout: Duration) -> Self {
        self.api_timeout.insert(api, timeout);
        self
    }

    /// Builds the options.
    pub fn build(self) -> crate::Result<ClientOptions> {
        let options = ClientOptions {
            node: None,
            nodes: self.nodes,
            node_pool_urls: self.node_pool_urls,
            network: self.network,
            mqtt_broker_options: self.mqtt_broker_options,
            local_pow: self.local_pow,
            node_sync_interval: self.node_sync_interval,
            node_sync_enabled: self.node_sync_enabled,
            request_timeout: self.request_timeout,
            api_timeout: self.api_timeout,
        };
        Ok(options)
    }
}

/// Each of the node APIs the wallet uses.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum Api {
    /// `get_tips` API
    GetTips,
    /// `post_message` API
    PostMessage,
    /// `get_output` API
    GetOutput,
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let t = match s {
            "GetTips" => Self::GetTips,
            "PostMessage" => Self::PostMessage,
            "GetOutput" => Self::GetOutput,
            _ => return Err(format!("unknown api kind `{}`", s)),
        };
        Ok(t)
    }
}

impl Serialize for Api {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(match self {
            Self::GetTips => "GetTips",
            Self::PostMessage => "PostMessage",
            Self::GetOutput => "GetOutput",
        })
    }
}

impl<'de> Deserialize<'de> for Api {
    fn deserialize<D>(deserializer: D) -> Result<Api, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;
        impl<'de> Visitor<'de> for StringVisitor {
            type Value = Api;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string representing an Api")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = Api::from_str(v).map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
        deserializer.deserialize_str(StringVisitor)
    }
}

impl Into<iota::Api> for Api {
    fn into(self) -> iota::Api {
        match self {
            Api::GetTips => iota::Api::GetTips,
            Api::PostMessage => iota::Api::PostMessage,
            Api::GetOutput => iota::Api::GetOutput,
        }
    }
}

/// The MQTT broker options.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrokerOptions {
    #[serde(rename = "automaticDisconnect")]
    pub(crate) automatic_disconnect: Option<bool>,
    pub(crate) timeout: Option<Duration>,
    #[serde(rename = "useWebsockets")]
    pub(crate) use_websockets: Option<bool>,
}

impl Into<iota::BrokerOptions> for BrokerOptions {
    fn into(self) -> iota::BrokerOptions {
        let mut options = iota::BrokerOptions::new();
        if let Some(automatic_disconnect) = self.automatic_disconnect {
            options = options.automatic_disconnect(automatic_disconnect);
        }
        if let Some(timeout) = self.timeout {
            options = options.timeout(timeout);
        }
        if let Some(use_websockets) = self.use_websockets {
            options = options.use_websockets(use_websockets);
        }
        options
    }
}

/// The client options type.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Getters)]
#[getset(get = "pub(crate)")]
pub struct ClientOptions {
    /// this option is here just to simplify usage from consumers using the deserialization
    node: Option<Url>,
    #[serde(default)]
    nodes: Vec<Url>,
    #[serde(rename = "nodePoolUrls", default)]
    node_pool_urls: Vec<Url>,
    network: Option<String>,
    #[serde(rename = "mqttBrokerOptions")]
    mqtt_broker_options: Option<BrokerOptions>,
    #[serde(rename = "localPow", default = "default_local_pow")]
    local_pow: bool,
    #[serde(rename = "nodeSyncInterval")]
    node_sync_interval: Option<Duration>,
    #[serde(rename = "nodeSyncEnabled", default = "default_node_sync_enabled")]
    node_sync_enabled: bool,
    #[serde(rename = "requestTimeout")]
    request_timeout: Option<Duration>,
    #[serde(rename = "apiTimeout", default)]
    api_timeout: HashMap<Api, Duration>,
}

impl ClientOptions {
    /// Gets a new client options builder instance.
    pub fn builder() -> ClientOptionsBuilder {
        ClientOptionsBuilder::new()
    }
}

impl Hash for ClientOptions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.nodes.hash(state);
        self.node_pool_urls.hash(state);
        self.network.hash(state);
        self.mqtt_broker_options.hash(state);
        self.local_pow.hash(state);
        self.request_timeout.hash(state);
    }
}

impl PartialEq for ClientOptions {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
            && self.nodes == other.nodes
            && self.node_pool_urls == other.node_pool_urls
            && self.network == other.network
            && self.mqtt_broker_options == other.mqtt_broker_options
            && self.local_pow == other.local_pow
            && self.request_timeout == other.request_timeout
    }
}

fn default_local_pow() -> bool {
    true
}

fn default_node_sync_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::ClientOptionsBuilder;

    #[test]
    fn single_node_valid_url() {
        let builder_res = ClientOptionsBuilder::new().with_node("https://api.lb-0.testnet.chrysalis2.com");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::new().with_node("some.invalid url");
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_valid_url() {
        let builder_res = ClientOptionsBuilder::new().with_nodes(&["https://api.lb-0.testnet.chrysalis2.com"]);
        assert!(builder_res.is_ok());
    }

    #[test]
    fn multi_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::new().with_nodes(&["some.invalid url"]);
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_empty() {
        let builder_res = ClientOptionsBuilder::new().with_nodes(&[]).unwrap().build();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn network_node_empty() {
        let builder_res = ClientOptionsBuilder::new().with_network("testnet2").build();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node() {
        let node = "https://api.lb-0.testnet.chrysalis2.com";
        let client = ClientOptionsBuilder::new().with_node(node).unwrap().build().unwrap();
        assert_eq!(client.nodes(), &super::convert_urls(&[node]).unwrap());
        assert!(client.network().is_none());
    }

    #[test]
    fn multi_node() {
        let nodes = ["https://api.lb-0.testnet.chrysalis2.com"];
        let client = ClientOptionsBuilder::new().with_nodes(&nodes).unwrap().build().unwrap();
        assert_eq!(client.nodes(), &super::convert_urls(&nodes).unwrap());
        assert!(client.network().is_none());
    }

    #[test]
    fn network() {
        let nodes = ["https://api.lb-0.testnet.chrysalis2.com"];
        let network = "testnet";
        let client = ClientOptionsBuilder::new()
            .with_network(network)
            .with_nodes(&nodes)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(client.nodes(), &super::convert_urls(&nodes).unwrap());
        assert_eq!(client.network(), &Some(network.to_string()));
    }

    #[tokio::test]
    async fn get_client() {
        let test_cases = vec![
            ClientOptionsBuilder::new()
                .with_node("https://api.lb-1.testnet.chrysalis2.com")
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_node("https://api.hornet-2.testnet.chrysalis2.com/")
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .with_network("mainnet")
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .with_network("testnet2")
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_network("testnet2")
                .with_nodes(&["https://api.hornet-3.testnet.chrysalis2.com/"])
                .unwrap()
                .build()
                .unwrap(),
        ];

        // assert that each different client_options create a new client instance
        for case in &test_cases {
            let len = super::instances().lock().await.len();
            super::get_client(&case).await;
            assert_eq!(super::instances().lock().await.len() - len, 1);
        }

        // assert that subsequent calls with options already initialized doesn't create new clients
        let len = super::instances().lock().await.len();
        for case in &test_cases {
            super::get_client(&case).await;
            assert_eq!(super::instances().lock().await.len(), len);
        }
    }
}

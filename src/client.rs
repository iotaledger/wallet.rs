// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota::client::{Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::RwLock;
use url::Url;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
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
            .with_mqtt_broker_options(
                options
                    .mqtt_broker_options()
                    .as_ref()
                    .map(|options| options.clone().into())
                    .unwrap_or_else(|| iota::BrokerOptions::new().automatic_disconnect(false)),
            )
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
            client_builder = client_builder.with_network(network);
        }

        if let Some(node_sync_interval) = options.node_sync_interval() {
            client_builder = client_builder.with_node_sync_interval(node_sync_interval.clone());
        }

        if !options.node_sync_enabled() {
            client_builder = client_builder.with_node_sync_disabled();
        }

        if let Some(request_timeout) = options.request_timeout() {
            client_builder = client_builder.with_request_timeout(request_timeout.clone());
        }

        for (api, timeout) in options.api_timeout() {
            client_builder = client_builder.with_api_timeout(api.clone().into(), timeout.clone());
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
    mqtt_broker_options: Option<BrokerOptions>,
    node_sync_interval: Option<Duration>,
    node_sync_enabled: bool,
    request_timeout: Option<Duration>,
    api_timeout: HashMap<Api, Duration>,
}

impl SingleNodeClientOptionsBuilder {
    fn new(node: &str) -> crate::Result<Self> {
        let node_url = Url::parse(node)?;
        let builder = Self {
            node: node_url,
            mqtt_broker_options: None,
            local_pow: default_local_pow(),
            node_sync_interval: None,
            node_sync_enabled: default_node_sync_enabled(),
            request_timeout: None,
            api_timeout: Default::default(),
        };
        Ok(builder)
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
    pub fn build(self) -> ClientOptions {
        ClientOptions {
            node: Some(self.node),
            nodes: None,
            network: None,
            mqtt_broker_options: self.mqtt_broker_options,
            local_pow: self.local_pow,
            node_sync_interval: self.node_sync_interval,
            node_sync_enabled: self.node_sync_enabled,
            request_timeout: self.request_timeout,
            api_timeout: self.api_timeout,
        }
    }
}

/// The options builder for a client connected to multiple nodes.
pub struct MultiNodeClientOptionsBuilder {
    nodes: Option<Vec<Url>>,
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

impl Default for MultiNodeClientOptionsBuilder {
    fn default() -> Self {
        Self {
            nodes: None,
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

impl MultiNodeClientOptionsBuilder {
    fn with_nodes(nodes: &[&str]) -> crate::Result<Self> {
        let nodes_urls = convert_urls(nodes)?;
        let builder = Self {
            nodes: Some(nodes_urls),
            ..Default::default()
        };
        Ok(builder)
    }

    fn with_network<N: Into<String>>(network: N) -> Self {
        Self {
            network: Some(network.into()),
            ..Default::default()
        }
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

/// The ClientOptions builder.
pub struct ClientOptionsBuilder;

impl ClientOptionsBuilder {
    /// Client connected to a single node.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::node("https://api.lb-0.testnet.chrysalis2.com")
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
    /// let client_options = ClientOptionsBuilder::nodes(&[
    ///     "https://api.lb-0.testnet.chrysalis2.com",
    ///     "https://api.hornet-1.testnet.chrysalis2.com/",
    /// ])
    /// .expect("invalid nodes URLs")
    /// .build();
    /// ```
    pub fn nodes(nodes: &[&str]) -> crate::Result<MultiNodeClientOptionsBuilder> {
        MultiNodeClientOptionsBuilder::with_nodes(nodes)
    }

    /// ClientOptions connected to the default Network pool.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::network("testnet2").build();
    /// ```
    pub fn network(network: &str) -> MultiNodeClientOptionsBuilder {
        MultiNodeClientOptionsBuilder::with_network(network)
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
                let value = Api::from_str(v).map_err(|e| serde::de::Error::custom(e))?;
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub(crate)")]
pub struct ClientOptions {
    node: Option<Url>,
    nodes: Option<Vec<Url>>,
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

impl Hash for ClientOptions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.nodes.hash(state);
        self.network.hash(state);
        self.mqtt_broker_options.hash(state);
        self.local_pow.hash(state);
        self.request_timeout.hash(state);
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
        let builder_res = ClientOptionsBuilder::node("https://api.lb-0.testnet.chrysalis2.com");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::node("some.invalid url");
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_valid_url() {
        let builder_res = ClientOptionsBuilder::nodes(&["https://api.lb-0.testnet.chrysalis2.com"]);
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
        let builder_res = ClientOptionsBuilder::network("testnet2").build();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_constructor() {
        let node = "https://api.lb-0.testnet.chrysalis2.com";
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
        let nodes = ["https://api.lb-0.testnet.chrysalis2.com"];
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
        let nodes = ["https://api.lb-0.testnet.chrysalis2.com"];
        let network = "testnet";
        let quorum_size = 50;
        let quorum_threshold = 0.9;
        let client = ClientOptionsBuilder::network(network)
            .quorum_size(quorum_size)
            .quorum_threshold(quorum_threshold)
            .nodes(&nodes)
            .unwrap()
            .build()
            .unwrap();
        assert!(client.node().is_none());
        assert_eq!(client.nodes(), &Some(super::convert_urls(&nodes).unwrap()));
        assert_eq!(client.network(), &Some(network.to_string()));
        assert_eq!(*client.quorum_size(), Some(quorum_size));
        assert!((*client.quorum_threshold() as f32 / 100.0 - quorum_threshold).abs() < f32::EPSILON);
    }

    #[test]
    fn get_client() {
        let test_cases = vec![
            ClientOptionsBuilder::node("https://api.lb-1.testnet.chrysalis2.com")
                .unwrap()
                .build(),
            ClientOptionsBuilder::node("https://api.hornet-2.testnet.chrysalis2.com/")
                .unwrap()
                .build(),
            ClientOptionsBuilder::nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://api.hornet-2.testnet.chrysalis2.com/"])
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .quorum_size(55)
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .network("mainnet")
                .build()
                .unwrap(),
            ClientOptionsBuilder::nodes(&["https://api.lb-1.testnet.chrysalis2.com"])
                .unwrap()
                .quorum_size(55)
                .quorum_threshold(0.6)
                .network("testnet2")
                .build()
                .unwrap(),
            ClientOptionsBuilder::network("testnet2")
                .nodes(&["https://api.hornet-3.testnet.chrysalis2.com/"])
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

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;

use iota_client::{node_manager::validate_url, Client, ClientBuilder};
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

pub(crate) async fn get_client(options: &ClientOptions) -> crate::Result<Arc<RwLock<Client>>> {
    let mut map = instances().lock().await;

    if !map.contains_key(options) {
        let mut client_builder = ClientBuilder::new()
            .with_mqtt_broker_options(
                options
                    .mqtt_broker_options()
                    .as_ref()
                    .map(|options| options.clone().into())
                    .unwrap_or_else(|| iota_client::BrokerOptions::new().automatic_disconnect(false)),
            )
            .with_local_pow(*options.local_pow())
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

        for node in options.nodes() {
            if !node.disabled {
                if let Some(auth) = &node.auth {
                    client_builder = client_builder.with_node_auth(
                        node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder.with_node(node.url.as_str()).unwrap();
                }
            }
        }

        if let Some(primary_node) = options.primary_node() {
            if !primary_node.disabled {
                if let Some(auth) = &primary_node.auth {
                    client_builder = client_builder.with_primary_node(
                        primary_node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder
                        .with_primary_node(primary_node.url.as_str(), None, None)
                        .unwrap();
                }
            }
        }

        if let Some(primary_pow_node) = options.primary_pow_node() {
            if !primary_pow_node.disabled {
                if let Some(auth) = &primary_pow_node.auth {
                    client_builder = client_builder.with_primary_pow_node(
                        primary_pow_node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder
                        .with_primary_pow_node(primary_pow_node.url.as_str(), None, None)
                        .unwrap();
                }
            }
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

        let client = client_builder.finish().await?;

        map.insert(options.clone(), Arc::new(RwLock::new(client)));
    }

    // safe to unwrap since we make sure the client exists on the block above
    let client = map.get(options).unwrap();

    Ok(client.clone())
}

/// Drops all clients.
pub async fn drop_all() {
    instances().lock().await.clear();
}

/// The options builder for a client connected to multiple nodes.
pub struct ClientOptionsBuilder {
    primary_node: Option<Node>,
    primary_pow_node: Option<Node>,
    nodes: Vec<Node>,
    node_pool_urls: Vec<Url>,
    network: Option<String>,
    mqtt_broker_options: Option<BrokerOptions>,
    mqtt_enabled: bool,
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
        .map(|node| match Url::parse(node) {
            Ok(url) => match validate_url(url) {
                Ok(url) => Some(url),
                Err(e) => {
                    err.replace(e);
                    None
                }
            },
            Err(e) => {
                err.replace(e.into());
                None
            }
        })
        .collect();

    if let Some(err) = err {
        Err(err.into())
    } else {
        // safe to unwrap: all URLs were parsed above
        let urls = urls.iter().map(|url| url.clone().unwrap()).collect();
        Ok(urls)
    }
}

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            primary_node: None,
            primary_pow_node: None,
            nodes: Vec::new(),
            node_pool_urls: Vec::new(),
            network: None,
            mqtt_broker_options: None,
            mqtt_enabled: default_mqtt_enabled(),
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

    /// Sets the primary node.
    pub fn with_primary_node(mut self, node: &str) -> crate::Result<Self> {
        self.primary_node.replace(validate_url(Url::parse(node)?)?.into());
        Ok(self)
    }

    /// Sets the primary PoW node.
    pub fn with_primary_pow_node(mut self, node: &str) -> crate::Result<Self> {
        self.primary_pow_node.replace(validate_url(Url::parse(node)?)?.into());
        Ok(self)
    }

    /// Sets the primary node with authentication.
    pub fn with_primary_node_auth(
        mut self,
        node: &str,
        jwt: Option<&str>,
        basic_auth_name_pwd: Option<(&str, &str)>,
    ) -> crate::Result<Self> {
        self.primary_node.replace(Node {
            url: validate_url(Url::parse(node)?)?,
            auth: NodeAuth {
                jwt: jwt.map(|r| r.to_string()),
                basic_auth_name_pwd: basic_auth_name_pwd.map(|(l, r)| (l.to_string(), r.to_string())),
            }
            .into(),
            disabled: false,
        });
        Ok(self)
    }

    /// Sets the primary pow node with authentication.
    pub fn with_primary_pow_node_auth(
        mut self,
        node: &str,
        jwt: Option<&str>,
        basic_auth_name_pwd: Option<(&str, &str)>,
    ) -> crate::Result<Self> {
        self.primary_pow_node.replace(Node {
            url: validate_url(Url::parse(node)?)?,
            auth: NodeAuth {
                jwt: jwt.map(|r| r.to_string()),
                basic_auth_name_pwd: basic_auth_name_pwd.map(|(l, r)| (l.to_string(), r.to_string())),
            }
            .into(),
            disabled: false,
        });
        Ok(self)
    }

    /// ClientOptions connected to a list of nodes.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new()
    ///     .with_nodes(&[
    ///         "https://api.lb-0.h.chrysalis-devnet.iota.cafe",
    ///         "https://api.lb-0.h.chrysalis-devnet.iota.cafe/",
    ///     ])
    ///     .expect("invalid nodes URLs")
    ///     .build();
    /// ```
    pub fn with_nodes(mut self, nodes: &[&str]) -> crate::Result<Self> {
        let nodes_urls = convert_urls(nodes)?;
        self.nodes
            .extend(nodes_urls.into_iter().map(|u| u.into()).collect::<Vec<Node>>());
        Ok(self)
    }

    /// Adds a node to the node list.
    pub fn with_node(mut self, node: &str) -> crate::Result<Self> {
        self.nodes.push(validate_url(Url::parse(node)?)?.into());
        Ok(self)
    }

    /// Adds a node with authentication to the node list.
    pub fn with_node_auth(
        mut self,
        node: &str,
        jwt: Option<&str>,
        basic_auth_name_pwd: Option<(&str, &str)>,
    ) -> crate::Result<Self> {
        self.nodes.push(Node {
            url: validate_url(Url::parse(node)?)?,
            auth: NodeAuth {
                jwt: jwt.map(|r| r.to_string()),
                basic_auth_name_pwd: basic_auth_name_pwd.map(|(l, r)| (l.to_string(), r.to_string())),
            }
            .into(),
            disabled: false,
        });
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
        self.network.replace(network.into());
        self
    }

    /// Set the node sync interval
    pub fn with_node_sync_interval(mut self, node_sync_interval: Duration) -> Self {
        self.node_sync_interval.replace(node_sync_interval);
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
        self.mqtt_broker_options.replace(options);
        self
    }

    /// Disables MQTT
    pub fn with_mqtt_disabled(mut self) -> Self {
        self.mqtt_enabled = false;
        self
    }

    /// Sets whether the PoW should be done locally or remotely.
    pub fn with_local_pow(mut self, local: bool) -> Self {
        self.local_pow = local;
        self
    }

    /// Sets the request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout.replace(timeout);
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
            primary_node: self.primary_node,
            primary_pow_node: self.primary_pow_node,
            nodes: self.nodes,
            node_pool_urls: self.node_pool_urls,
            network: self.network,
            mqtt_broker_options: self.mqtt_broker_options,
            mqtt_enabled: self.mqtt_enabled,
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
    /// `get_balance` API
    GetBalance,
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let t = match s {
            "GetTips" => Self::GetTips,
            "PostMessage" => Self::PostMessage,
            "GetOutput" => Self::GetOutput,
            "GetBalance" => Self::GetBalance,
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
            Self::GetBalance => "GetBalance",
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

impl From<Api> for iota_client::Api {
    fn from(api: Api) -> iota_client::Api {
        match api {
            Api::GetTips => iota_client::Api::GetTips,
            Api::PostMessage => iota_client::Api::PostMessage,
            Api::GetOutput => iota_client::Api::GetOutput,
            Api::GetBalance => iota_client::Api::GetBalance,
        }
    }
}

/// The MQTT broker options.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrokerOptions {
    // We need to use `pub` here or these is no way to let the user create BrokerOptions
    #[serde(rename = "automaticDisconnect")]
    /// Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not.
    pub automatic_disconnect: Option<bool>,
    /// timeout of the mqtt broker.
    pub timeout: Option<Duration>,
    /// Defines if websockets should be used (true) or TCP (false)
    #[serde(rename = "useWs")]
    pub use_ws: Option<bool>,
    /// Defines the port to be used for the MQTT connection
    pub port: Option<u16>,
    /// Defines the maximum reconnection attempts before it returns an error
    #[serde(rename = "maxReconnectionAttempts")]
    pub max_reconnection_attempts: Option<usize>,
}

impl From<BrokerOptions> for iota_client::BrokerOptions {
    fn from(value: BrokerOptions) -> iota_client::BrokerOptions {
        let mut options = iota_client::BrokerOptions::new();
        if let Some(automatic_disconnect) = value.automatic_disconnect {
            options = options.automatic_disconnect(automatic_disconnect);
        }
        if let Some(timeout) = value.timeout {
            options = options.timeout(timeout);
        }
        if let Some(use_ws) = value.use_ws {
            options = options.use_ws(use_ws);
        }
        if let Some(port) = value.port {
            options = options.port(port);
        }
        if let Some(max_reconnection_attempts) = value.max_reconnection_attempts {
            options = options.max_reconnection_attempts(max_reconnection_attempts);
        }
        options
    }
}

/// Node authentication object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeAuth {
    /// JWT.
    pub jwt: Option<String>,
    /// Username and password.
    pub basic_auth_name_pwd: Option<(String, String)>,
}

/// Node definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub(crate)")]
pub struct Node {
    /// Node url.
    pub url: Url,
    /// Node auth options.
    pub auth: Option<NodeAuth>,
    /// Whether the node is disabled or not.
    #[serde(default)]
    pub disabled: bool,
}

impl From<Url> for Node {
    fn from(url: Url) -> Self {
        Self {
            url,
            auth: None,
            disabled: false,
        }
    }
}

/// The client options type.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Getters)]
/// Need to set the get methods to be public for binding
#[getset(get = "pub")]
pub struct ClientOptions {
    /// The primary node to connect to.
    #[serde(rename = "node")] // here just for DB compatibility; can be changed when migrations are implemented
    primary_node: Option<Node>,
    /// The primary PoW node to connect to.
    #[serde(rename = "primaryPoWNode")]
    primary_pow_node: Option<Node>,
    /// The nodes to connect to.
    #[serde(default)]
    nodes: Vec<Node>,
    /// The node pool urls.
    #[serde(rename = "nodePoolUrls", default)]
    node_pool_urls: Vec<Url>,
    /// The network string.
    network: Option<String>,
    /// The MQTT broker options.
    #[serde(rename = "mqttBrokerOptions")]
    mqtt_broker_options: Option<BrokerOptions>,
    /// Enable local proof-of-work or not.
    #[serde(rename = "localPow", default = "default_local_pow")]
    local_pow: bool,
    /// The node sync interval.
    #[serde(rename = "nodeSyncInterval")]
    node_sync_interval: Option<Duration>,
    /// Enable node synchronization or not.
    #[serde(rename = "nodeSyncEnabled", default = "default_node_sync_enabled")]
    node_sync_enabled: bool,
    /// Enable mqtt or not.
    #[serde(rename = "mqttEnabled", default = "default_mqtt_enabled")]
    mqtt_enabled: bool,
    /// The request timeout.
    #[serde(rename = "requestTimeout")]
    request_timeout: Option<Duration>,
    /// The API timeout.
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
        self.primary_node.hash(state);
        self.primary_pow_node.hash(state);
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
        self.primary_node == other.primary_node
            && self.primary_pow_node == other.primary_pow_node
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

fn default_mqtt_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::ClientOptionsBuilder;

    #[test]
    fn primary_node_valid_url() {
        let builder_res =
            ClientOptionsBuilder::new().with_primary_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn primary_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::new().with_primary_node("some.invalid url");
        assert!(builder_res.is_err());
    }
    #[test]
    fn primary_pow_node_valid_url() {
        let builder_res =
            ClientOptionsBuilder::new().with_primary_pow_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn primary_pow_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::new().with_primary_pow_node("some.invalid url");
        assert!(builder_res.is_err());
    }

    #[test]
    fn single_node_valid_url() {
        let builder_res = ClientOptionsBuilder::new().with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe");
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node_invalid_url() {
        let builder_res = ClientOptionsBuilder::new().with_node("some.invalid url");
        assert!(builder_res.is_err());
    }

    #[test]
    fn multi_node_valid_url() {
        let builder_res = ClientOptionsBuilder::new().with_nodes(&["https://api.lb-0.h.chrysalis-devnet.iota.cafe"]);
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
        let node = "https://api.lb-0.h.chrysalis-devnet.iota.cafe";
        let client = ClientOptionsBuilder::new().with_node(node).unwrap().build().unwrap();
        assert_eq!(
            client.nodes(),
            &super::convert_urls(&[node])
                .unwrap()
                .into_iter()
                .map(|u| u.into())
                .collect::<Vec<super::Node>>()
        );
        assert!(client.network().is_none());
    }

    #[test]
    fn multi_node() {
        let nodes = ["https://api.lb-0.h.chrysalis-devnet.iota.cafe"];
        let client = ClientOptionsBuilder::new().with_nodes(&nodes).unwrap().build().unwrap();
        assert_eq!(
            client.nodes(),
            &super::convert_urls(&nodes)
                .unwrap()
                .into_iter()
                .map(|u| u.into())
                .collect::<Vec<super::Node>>()
        );
        assert!(client.network().is_none());
    }

    #[test]
    fn network() {
        let nodes = ["https://api.lb-0.h.chrysalis-devnet.iota.cafe"];
        let network = "testnet";
        let client = ClientOptionsBuilder::new()
            .with_network(network)
            .with_nodes(&nodes)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(
            client.nodes(),
            &super::convert_urls(&nodes)
                .unwrap()
                .into_iter()
                .map(|u| u.into())
                .collect::<Vec<super::Node>>()
        );
        assert_eq!(client.network(), &Some(network.to_string()));
    }

    #[tokio::test]
    async fn get_client() {
        let test_cases = vec![
            ClientOptionsBuilder::new()
                .with_node("https://api.lb-1.h.chrysalis-devnet.iota.cafe")
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe/")
                .unwrap()
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.h.chrysalis-devnet.iota.cafe"])
                .unwrap()
                .with_network("mainnet")
                .build()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.h.chrysalis-devnet.iota.cafe"])
                .unwrap()
                .with_network("testnet2")
                .build()
                .unwrap(),
        ];

        // assert that each different client_options create a new client instance
        for case in &test_cases {
            let len = super::instances().lock().await.len();
            super::get_client(case).await.unwrap();
            assert_eq!(super::instances().lock().await.len() - len, 1);
        }

        // assert that subsequent calls with options already initialized doesn't create new clients
        let len = super::instances().lock().await.len();
        for case in &test_cases {
            super::get_client(case).await.unwrap();
            assert_eq!(super::instances().lock().await.len(), len);
        }
    }
}

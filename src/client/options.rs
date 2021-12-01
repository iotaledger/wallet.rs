// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota_client::node_manager::validate_url;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::client::{
    api::Api,
    node::{Node, NodeAuth},
};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    time::Duration,
};

/// The client options type.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Getters)]
/// Need to set the get methods to be public for binding
#[getset(get = "pub")]
pub struct ClientOptions {
    /// The primary node to connect to.
    #[serde(rename = "node")]
    // here just for DB compatibility; can be changed when migrations are implemented
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
    // /// The MQTT broker options.
    // #[serde(rename = "mqttBrokerOptions")]
    // mqtt_broker_options: Option<BrokerOptions>,
    // /// Enable mqtt or not.
    // #[serde(rename = "mqttEnabled", default = "default_mqtt_enabled")]
    // mqtt_enabled: bool,
    /// Enable local proof-of-work or not.
    #[serde(rename = "localPow", default = "default_local_pow")]
    local_pow: bool,
    /// The node sync interval.
    #[serde(rename = "nodeSyncInterval")]
    node_sync_interval: Option<Duration>,
    /// Enable node synchronization or not.
    #[serde(rename = "nodeSyncEnabled", default = "default_node_sync_enabled")]
    node_sync_enabled: bool,
    /// The request timeout.
    #[serde(rename = "requestTimeout")]
    request_timeout: Option<Duration>,
    /// The API timeout.
    #[serde(rename = "apiTimeout", default)]
    api_timeout: HashMap<Api, Duration>,
}

pub fn default_local_pow() -> bool {
    true
}
pub fn default_node_sync_enabled() -> bool {
    true
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
        // self.mqtt_broker_options.hash(state);
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
            // && self.mqtt_broker_options == other.mqtt_broker_options
            && self.local_pow == other.local_pow
            && self.request_timeout == other.request_timeout
    }
}

/// The options builder for a client connected to multiple nodes.
pub struct ClientOptionsBuilder {
    primary_node: Option<Node>,
    primary_pow_node: Option<Node>,
    nodes: Vec<Node>,
    node_pool_urls: Vec<Url>,
    network: Option<String>,
    // mqtt_broker_options: Option<BrokerOptions>,
    // mqtt_enabled: bool,
    local_pow: bool,
    node_sync_interval: Option<Duration>,
    node_sync_enabled: bool,
    request_timeout: Option<Duration>,
    api_timeout: HashMap<Api, Duration>,
}

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            primary_node: None,
            primary_pow_node: None,
            nodes: Vec::new(),
            node_pool_urls: Vec::new(),
            network: None,
            // mqtt_broker_options: None,
            // mqtt_enabled: default_mqtt_enabled(),
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
    /// use wallet_core::client::options::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new()
    ///     .with_nodes(&[
    ///         "https://api.lb-0.h.chrysalis-devnet.iota.cafe",
    ///         "https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe/",
    ///     ])
    ///     .expect("invalid nodes URLs")
    ///     .finish();
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
        // todo: change it to a vec or something else so it works for other languages
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
    /// use wallet_core::client::options::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new().with_network("testnet2").finish();
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

    // /// Sets the MQTT broker options.
    // pub fn with_mqtt_mqtt_broker_options(mut self, options: BrokerOptions) -> Self {
    //     self.mqtt_broker_options.replace(options);
    //     self
    // }

    // /// Sets the MQTT broker options.
    // pub fn with_mqtt_disabled(mut self) -> Self {
    //     self.mqtt_enabled = false;
    //     self
    // }

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
    pub fn finish(self) -> crate::Result<ClientOptions> {
        let options = ClientOptions {
            primary_node: self.primary_node,
            primary_pow_node: self.primary_pow_node,
            nodes: self.nodes,
            node_pool_urls: self.node_pool_urls,
            network: self.network,
            // mqtt_broker_options: self.mqtt_broker_options,
            // mqtt_enabled: self.mqtt_enabled,
            local_pow: self.local_pow,
            node_sync_interval: self.node_sync_interval,
            node_sync_enabled: self.node_sync_enabled,
            request_timeout: self.request_timeout,
            api_timeout: self.api_timeout,
        };
        Ok(options)
    }
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
        let builder_res = ClientOptionsBuilder::new().with_nodes(&[]).unwrap().finish();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn network_node_empty() {
        let builder_res = ClientOptionsBuilder::new().with_network("testnet2").finish();
        assert!(builder_res.is_ok());
    }

    #[test]
    fn single_node() {
        let node = "https://api.lb-0.h.chrysalis-devnet.iota.cafe";
        let client = ClientOptionsBuilder::new().with_node(node).unwrap().finish().unwrap();
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
        let client = ClientOptionsBuilder::new()
            .with_nodes(&nodes)
            .unwrap()
            .finish()
            .unwrap();
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
            .finish()
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
                .finish()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe/")
                .unwrap()
                .finish()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.h.chrysalis-devnet.iota.cafe"])
                .unwrap()
                .with_network("mainnet")
                .finish()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_nodes(&["https://api.lb-1.h.chrysalis-devnet.iota.cafe"])
                .unwrap()
                .with_network("testnet2")
                .finish()
                .unwrap(),
            ClientOptionsBuilder::new()
                .with_network("testnet2")
                .with_nodes(&["https://api.fat-hornet-0.h.chrysalis-devnet.iota.cafe/"])
                .unwrap()
                .finish()
                .unwrap(),
        ];

        for case in &test_cases {
            crate::client::set_client(case.clone()).await.unwrap();
            crate::client::get_client().await.unwrap();
        }
    }
}

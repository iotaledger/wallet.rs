// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_wallet::client::{
    Api as RustApi, BrokerOptions as RustBrokerOptions, ClientOptions as RustClientOptions, Node as RustNode,
    NodeAuth as RustNodeAuth,
};
use std::{
    collections::HashMap,
    convert::{From, Into},
    str::FromStr,
    time::Duration,
};
use url::Url;

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct NodeAuth {
    username: String,
    password: String,
}

impl From<NodeAuth> for RustNodeAuth {
    fn from(auth: NodeAuth) -> RustNodeAuth {
        RustNodeAuth {
            username: auth.username,
            password: auth.password,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Node {
    url: String,
    auth: Option<NodeAuth>,
}

impl From<Node> for RustNode {
    fn from(node: Node) -> Self {
        Self {
            url: Url::parse(&node.url).expect("invalid url"),
            auth: node.auth.map(|a| a.into()),
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct ClientOptions {
    pub nodes: Option<Vec<Node>>,
    pub node_pool_urls: Option<Vec<String>>,
    pub network: Option<String>,
    pub mqtt_broker_options: Option<BrokerOptions>,
    pub local_pow: Option<bool>,
    /// in mllisecond
    pub node_sync_interval: Option<u64>,
    pub node_sync_enabled: Option<bool>,
    /// in mllisecond
    pub request_timeout: Option<u64>,
    pub api_timeout: Option<HashMap<String, u64>>,
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct BrokerOptions {
    /// automatic disconnect or not
    pub automatic_disconnect: Option<bool>,
    /// broker timeout in secs
    pub timeout: Option<u64>,
    /// use websockets or not
    pub use_websockets: Option<bool>,
}

impl From<BrokerOptions> for RustBrokerOptions {
    fn from(broker_options: BrokerOptions) -> Self {
        Self {
            automatic_disconnect: broker_options.automatic_disconnect,
            timeout: if let Some(timeout) = broker_options.timeout {
                Some(Duration::from_secs(timeout))
            } else {
                None
            },
            use_websockets: broker_options.use_websockets.unwrap_or(false),
        }
    }
}

impl From<ClientOptions> for RustClientOptions {
    fn from(client_options: ClientOptions) -> Self {
        let mut builder = RustClientOptions::builder();
        if let Some(nodes) = client_options.nodes {
            for node in nodes {
                let node: RustNode = node.into();
                if let Some(auth) = node.auth {
                    builder = builder
                        .with_node_auth(node.url.as_str(), &auth.username, &auth.password)
                        .unwrap();
                } else {
                    builder = builder.with_node(node.url.as_str()).unwrap();
                }
            }
        }
        if let Some(node_pool_urls) = client_options.node_pool_urls {
            builder = builder
                .with_node_pool_urls(&(node_pool_urls.iter().map(|url| &url[..]).collect::<Vec<&str>>()))
                .unwrap();
        }
        if let Some(network) = client_options.network {
            builder = builder.with_network(network);
        }
        if let Some(broker_options) = client_options.mqtt_broker_options {
            builder = builder.with_mqtt_mqtt_broker_options(broker_options.into());
        }
        if let Some(local_pow) = client_options.local_pow {
            builder = builder.with_local_pow(local_pow);
        }
        if let Some(node_sync_interval) = client_options.node_sync_interval {
            builder = builder.with_node_sync_interval(Duration::from_millis(node_sync_interval));
        }
        if !client_options.node_sync_enabled.unwrap_or(true) {
            builder = builder.with_node_sync_disabled();
        }
        if let Some(request_timeout) = client_options.request_timeout {
            builder = builder.with_request_timeout(Duration::from_millis(request_timeout));
        }
        if let Some(api_timeout) = client_options.api_timeout {
            for (api, timeout) in api_timeout {
                builder =
                    builder.with_api_timeout(RustApi::from_str(api.as_str()).unwrap(), Duration::from_millis(timeout));
            }
        }
        builder.build().unwrap()
    }
}

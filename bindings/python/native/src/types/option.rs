// Copyright 2020 IOTA Stiftung
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
    jwt: Option<String>,
    basic_auth_name_pwd: Option<(String, String)>,
}

impl From<NodeAuth> for RustNodeAuth {
    fn from(auth: NodeAuth) -> Self {
        Self {
            jwt: auth.jwt,
            basic_auth_name_pwd: auth.basic_auth_name_pwd,
        }
    }
}

impl From<RustNodeAuth> for NodeAuth {
    fn from(auth: RustNodeAuth) -> Self {
        Self {
            jwt: auth.jwt,
            basic_auth_name_pwd: auth.basic_auth_name_pwd,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Node {
    url: String,
    auth: Option<NodeAuth>,
    disabled: bool,
}

impl From<Node> for RustNode {
    fn from(node: Node) -> Self {
        Self {
            url: Url::parse(&node.url).expect("invalid url"),
            auth: node.auth.map(|a| a.into()),
            disabled: node.disabled,
        }
    }
}

impl From<RustNode> for Node {
    fn from(node: RustNode) -> Self {
        Self {
            url: node.url.as_str().to_string(),
            auth: node.auth.map(|auth| auth.into()),
            disabled: node.disabled,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct ClientOptions {
    pub primary_node: Option<Node>,
    pub primary_pow_node: Option<Node>,
    pub nodes: Option<Vec<Node>>,
    pub node_pool_urls: Option<Vec<String>>,
    pub network: Option<String>,
    pub mqtt_broker_options: Option<BrokerOptions>,
    pub local_pow: Option<bool>,
    /// in mllisecond
    pub node_sync_interval: Option<u64>,
    pub node_sync_enabled: Option<bool>,
    pub mqtt_enabled: Option<bool>,
    /// in mllisecond
    pub request_timeout: Option<u64>,
    /// in mllisecond
    pub api_timeout: Option<HashMap<String, u64>>,
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct BrokerOptions {
    /// automatic disconnect or not
    pub automatic_disconnect: Option<bool>,
    /// broker timeout in secs
    pub timeout: Option<u64>,
    /// Defines if websockets should be used (true) or TCP (false)
    pub use_ws: Option<bool>,
    /// Defines the port to be used for the MQTT connection
    pub port: Option<u16>,
    /// Defines the maximum reconnection attempts before it returns an error
    pub max_reconnection_attempts: Option<usize>,
}

impl From<BrokerOptions> for RustBrokerOptions {
    fn from(broker_options: BrokerOptions) -> Self {
        Self {
            automatic_disconnect: broker_options.automatic_disconnect,
            timeout: broker_options.timeout.map(Duration::from_secs),
            use_ws: broker_options.use_ws,
            port: broker_options.port,
            max_reconnection_attempts: broker_options.max_reconnection_attempts,
        }
    }
}

impl From<ClientOptions> for RustClientOptions {
    fn from(client_options: ClientOptions) -> Self {
        let mut builder = RustClientOptions::builder();
        if let Some(primary_node) = client_options.primary_node {
            let primary_node: RustNode = primary_node.into();
            if let Some(auth) = primary_node.auth {
                builder = builder
                    .with_primary_node_auth(
                        primary_node.url.as_str(),
                        auth.jwt.as_deref(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )
                    .expect("with_primary_node_auth failed");
            } else {
                builder = builder
                    .with_primary_node(primary_node.url.as_str())
                    .expect("with_primary_node failed");
            }
        }
        if let Some(primary_pow_node) = client_options.primary_pow_node {
            let primary_pow_node: RustNode = primary_pow_node.into();
            if let Some(auth) = primary_pow_node.auth {
                builder = builder
                    .with_primary_pow_node_auth(
                        primary_pow_node.url.as_str(),
                        auth.jwt.as_deref(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )
                    .expect("with_primary_pow_node_auth failed");
            } else {
                builder = builder
                    .with_primary_pow_node(primary_pow_node.url.as_str())
                    .expect("with_primary_pow_node failed");
            }
        }
        if let Some(nodes) = client_options.nodes {
            for node in nodes {
                let node: RustNode = node.into();
                if let Some(auth) = node.auth {
                    builder = builder
                        .with_node_auth(
                            node.url.as_str(),
                            auth.jwt.as_deref(),
                            auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                        )
                        .expect("with_node_auth failed");
                } else {
                    builder = builder.with_node(node.url.as_str()).expect("with_node failed");
                }
            }
        }
        if let Some(node_pool_urls) = client_options.node_pool_urls {
            builder = builder
                .with_node_pool_urls(&(node_pool_urls.iter().map(|url| &url[..]).collect::<Vec<&str>>()))
                .expect("with_node_pool_urls failed");
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
        if !client_options.mqtt_enabled.unwrap_or(true) {
            builder = builder.with_mqtt_disabled();
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

impl From<&RustBrokerOptions> for BrokerOptions {
    fn from(broker_options: &RustBrokerOptions) -> Self {
        Self {
            automatic_disconnect: broker_options.automatic_disconnect,
            timeout: broker_options.timeout.map(|s| s.as_secs()),
            use_ws: broker_options.use_ws,
            port: broker_options.port,
            max_reconnection_attempts: broker_options.max_reconnection_attempts,
        }
    }
}

impl From<RustClientOptions> for ClientOptions {
    fn from(client_options: RustClientOptions) -> Self {
        Self {
            primary_node: client_options.primary_node().as_ref().map(|n| n.clone().into()),
            primary_pow_node: client_options.primary_pow_node().as_ref().map(|n| n.clone().into()),
            nodes: Some(client_options.nodes().iter().map(|s| s.clone().into()).collect()),
            node_pool_urls: Some(
                client_options
                    .node_pool_urls()
                    .iter()
                    .map(|s| s.as_str().to_string())
                    .collect(),
            ),
            network: client_options.network().as_ref().map(|s| s.to_string()),
            mqtt_enabled: Some(*client_options.mqtt_enabled()),
            mqtt_broker_options: client_options
                .mqtt_broker_options()
                .as_ref()
                .map(|options| options.into()),
            local_pow: Some(*client_options.local_pow()),
            node_sync_interval: client_options.node_sync_interval().map(duration_to_millisec),
            node_sync_enabled: Some(*client_options.node_sync_enabled()),
            request_timeout: client_options.request_timeout().map(duration_to_millisec),
            api_timeout: {
                let mut map: HashMap<String, u64> = HashMap::new();
                for (api, s) in client_options.api_timeout().iter() {
                    let api = match api {
                        RustApi::GetTips => "GetTips",
                        RustApi::PostMessage => "PostMessage",
                        RustApi::GetOutput => "GetOutput",
                    };
                    map.insert(api.to_string(), duration_to_millisec(*s));
                }
                Some(map)
            },
        }
    }
}

/// Helper function of casting duration to millisec
fn duration_to_millisec(s: Duration) -> u64 {
    s.as_secs() * 1000 + s.subsec_nanos() as u64 / 1_000_000
}

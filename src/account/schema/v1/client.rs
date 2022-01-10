// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

pub(in crate::account::schema) type Url = String;

#[derive(Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub(in crate::account::schema) enum Api {
    /// `get_tips` API
    GetTips,
    /// `post_message` API
    PostMessage,
    /// `get_output` API
    GetOutput,
    /// `get_balance` API
    GetBalance,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(in crate::account::schema) struct NodeAuth {
    /// JWT.
    pub(in crate::account::schema) jwt: Option<String>,
    /// Username and password.
    pub(in crate::account::schema) basic_auth_name_pwd: Option<(String, String)>,
}

/// Node definition.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::account::schema) struct Node {
    /// Node url.
    pub(in crate::account::schema) url: Url,
    /// Node auth options.
    pub(in crate::account::schema) auth: Option<NodeAuth>,
    /// Whether the node is disabled or not.
    #[serde(default)]
    pub(in crate::account::schema) disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::account::schema) struct BrokerOptions {
    // We need to use `pub(in crate::account::schema)` here or these is no way to let the user create BrokerOptions
    /// Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not.
    pub(in crate::account::schema) automatic_disconnect: Option<bool>,
    /// timeout of the mqtt broker.
    pub(in crate::account::schema) timeout: Option<Duration>,
    /// Defines if websockets should be used (true) or TCP (false)
    pub(in crate::account::schema) use_ws: Option<bool>,
    /// Defines the port to be used for the MQTT connection
    pub(in crate::account::schema) port: Option<u16>,
    /// Defines the maximum reconnection attempts before it returns an error
    pub(in crate::account::schema) max_reconnection_attempts: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::account::schema) struct ClientOptions {
    /// The primary node to connect to.
    #[serde(rename = "node")] // here just for DB compatibility; can be changed when migrations are implemented
    pub(in crate::account::schema) primary_node: Option<Node>,
    /// The primary PoW node to connect to.
    #[serde(rename = "primaryPoWNode")]
    pub(in crate::account::schema) primary_pow_node: Option<Node>,
    /// The nodes to connect to.
    #[serde(default)]
    pub(in crate::account::schema) nodes: Vec<Node>,
    /// The node pool urls.
    pub(in crate::account::schema) node_pool_urls: Vec<Url>,
    /// The network string.
    pub(in crate::account::schema) network: Option<String>,
    /// The MQTT broker options.
    pub(in crate::account::schema) mqtt_broker_options: Option<BrokerOptions>,
    /// Enable local proof-of-work or not.
    pub(in crate::account::schema) local_pow: bool,
    /// The node sync interval.
    pub(in crate::account::schema) node_sync_interval: Option<Duration>,
    /// Enable node synchronization or not.
    pub(in crate::account::schema) node_sync_enabled: bool,
    /// Enable mqtt or not.
    pub(in crate::account::schema) mqtt_enabled: bool,
    /// The request timeout.
    pub(in crate::account::schema) request_timeout: Option<Duration>,
    /// The API timeout.
    pub(in crate::account::schema) api_timeout: HashMap<Api, Duration>,
}

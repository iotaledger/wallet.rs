// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::client::{Api, BrokerOptions, ClientOptions, Node};
use serde::Deserialize;
use url::Url;

use std::{collections::HashMap, time::Duration};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum NodeDto {
    Url(Url),
    Node(Node),
}

impl From<NodeDto> for Node {
    fn from(node: NodeDto) -> Self {
        match node {
            NodeDto::Url(url) => url.into(),
            NodeDto::Node(node) => node,
        }
    }
}

fn default_node_sync_enabled() -> bool {
    true
}

fn default_mqtt_enabled() -> bool {
    true
}

fn default_local_pow() -> bool {
    true
}

#[derive(Deserialize)]
pub struct ClientOptionsDto {
    #[serde(rename = "primaryNode")]
    primary_node: Option<NodeDto>,
    #[serde(rename = "primaryPoWNode")]
    primary_pow_node: Option<NodeDto>,
    node: Option<NodeDto>,
    #[serde(default)]
    nodes: Vec<NodeDto>,
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
    #[serde(rename = "mqttEnabled", default = "default_mqtt_enabled")]
    mqtt_enabled: bool,
    #[serde(rename = "requestTimeout")]
    request_timeout: Option<Duration>,
    #[serde(rename = "apiTimeout", default)]
    api_timeout: HashMap<Api, Duration>,
}

macro_rules! bind_client_option {
    ($builder:expr, $arg:expr, $fn_name:ident) => {{
        let mut builder = $builder;
        if let Some(value) = $arg {
            builder = builder.$fn_name(value);
        }
        builder
    }};
}

impl From<ClientOptionsDto> for ClientOptions {
    fn from(options: ClientOptionsDto) -> Self {
        let mut client_builder = Self::builder()
            .with_node_pool_urls(
                &options
                    .node_pool_urls
                    .iter()
                    .map(|url| url.as_str())
                    .collect::<Vec<&str>>()[..],
            )
            .unwrap()
            .with_local_pow(options.local_pow);
        if let Some(primary_node) = options.primary_node {
            let node: Node = primary_node.into();
            if let Some(auth) = node.auth {
                client_builder = client_builder
                    .with_primary_node_auth(
                        node.url.as_str(),
                        auth.jwt.as_deref(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )
                    .unwrap();
            } else {
                client_builder = client_builder.with_primary_node(node.url.as_str()).unwrap();
            }
        }
        if let Some(primary_pow_node) = options.primary_pow_node {
            let node: Node = primary_pow_node.into();
            if let Some(auth) = node.auth {
                client_builder = client_builder
                    .with_primary_pow_node_auth(
                        node.url.as_str(),
                        auth.jwt.as_deref(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )
                    .unwrap();
            } else {
                client_builder = client_builder.with_primary_pow_node(node.url.as_str()).unwrap();
            }
        }
        let mut nodes = options.nodes;
        if let Some(node) = options.node {
            nodes.push(node);
        }
        for node in nodes {
            let node: Node = node.into();
            if let Some(auth) = node.auth {
                client_builder = client_builder
                    .with_node_auth(
                        node.url.as_str(),
                        auth.jwt.as_deref(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )
                    .unwrap();
            } else {
                client_builder = client_builder.with_node(node.url.as_str()).unwrap();
            }
        }

        client_builder = bind_client_option!(client_builder, options.network, with_network);
        client_builder = bind_client_option!(
            client_builder,
            options.mqtt_broker_options,
            with_mqtt_mqtt_broker_options
        );
        client_builder = bind_client_option!(client_builder, options.node_sync_interval, with_node_sync_interval);

        if !options.node_sync_enabled {
            client_builder = client_builder.with_node_sync_disabled();
        }

        if !options.mqtt_enabled {
            client_builder = client_builder.with_mqtt_disabled();
        }

        client_builder = bind_client_option!(client_builder, options.request_timeout, with_request_timeout);

        for (api, timeout) in options.api_timeout {
            client_builder = client_builder.with_api_timeout(api, timeout);
        }

        client_builder.build().unwrap()
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
// pub(crate) mod mqtt;
pub mod node;
pub mod options;

use iota_client::{node_manager::validate_url, Client, ClientBuilder};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use url::Url;

use crate::client::options::ClientOptions;

use std::sync::Arc;

type ClientInstance = Arc<RwLock<Option<Arc<Client>>>>;

/// Gets the client instance.
fn client_instance() -> &'static ClientInstance {
    static CLIENT_INSTANCE: Lazy<ClientInstance> = Lazy::new(Default::default);
    &CLIENT_INSTANCE
}

pub(crate) async fn get_client() -> crate::Result<Arc<Client>> {
    let lock = client_instance().read().await;
    if let Some(client) = &*lock {
        Ok(client.clone())
    } else {
        Err(crate::Error::ClientNotSet)
    }
}

pub(crate) async fn set_client(options: ClientOptions) -> crate::Result<()> {
    let mut client_builder = ClientBuilder::new()
        // .with_mqtt_broker_options(
        //     options
        //         .mqtt_broker_options()
        //         .as_ref()
        //         .map(|options| options.clone().into())
        //         .unwrap_or_else(|| {
        //             iota_client::BrokerOptions::new().automatic_disconnect(false)
        //         }),
        // )
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

    let mut client_instance = client_instance().write().await;
    client_instance.replace(Arc::new(client));

    Ok(())
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

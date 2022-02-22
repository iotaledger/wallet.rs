// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use iota_client::{Client, ClientBuilder};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

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

pub(crate) async fn set_client(client_builder: ClientBuilder) -> crate::Result<()> {
    let client = client_builder.finish().await?;

    let mut client_instance = client_instance().write().await;
    client_instance.replace(Arc::new(client));

    Ok(())
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc, time::Duration};

use iota_wallet::client::{Api, BrokerOptions as BrokerOptionsRust, ClientOptions as ClientOptionsRust};

use crate::Result;

pub struct BrokerOptions {
    builder: Rc<RefCell<Option<BrokerOptionsRust>>>,
}

impl BrokerOptions {
    pub fn new() -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(BrokerOptionsRust {
                automatic_disconnect: None,
                timeout: None,
                use_ws: None,
                port: None,
                max_reconnection_attempts: None,
            }))),
        }
    }

    fn new_with(options: BrokerOptionsRust) -> BrokerOptions {
        Self {
            builder: Rc::new(RefCell::new(Option::from(options))),
        }
    }

    pub fn automatic_disconnect(&self, disconnect: bool) -> BrokerOptions {
        let mut builder = self.builder.borrow_mut().take().unwrap();
        builder.automatic_disconnect = Some(disconnect);
        BrokerOptions::new_with(builder)
    }

    pub fn timeout(&self, timeout: Duration) -> BrokerOptions {
        let mut builder = self.builder.borrow_mut().take().unwrap();
        builder.timeout = Some(timeout);
        BrokerOptions::new_with(builder)
    }
    pub fn use_ws(&self, use_ws: bool) -> BrokerOptions {
        let mut builder = self.builder.borrow_mut().take().unwrap();
        builder.use_ws = Some(use_ws);
        BrokerOptions::new_with(builder)
    }
    pub fn port(&self, port: u16) -> BrokerOptions {
        let mut builder = self.builder.borrow_mut().take().unwrap();
        builder.port = Some(port);
        BrokerOptions::new_with(builder)
    }
    pub fn max_reconnection_attempts(&self, max_reconnection_attempts: usize) -> BrokerOptions {
        let mut builder = self.builder.borrow_mut().take().unwrap();
        builder.max_reconnection_attempts = Some(max_reconnection_attempts);
        BrokerOptions::new_with(builder)
    }
}

pub struct ClientOptions {
    options: ClientOptionsRust,
}

impl core::fmt::Display for ClientOptions {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.options)
    }
}

impl From<ClientOptionsRust> for ClientOptions {
    fn from(options: ClientOptionsRust) -> Self {
        Self { options }
    }
}

impl ClientOptions {
    pub fn to_inner(self) -> ClientOptionsRust {
        self.options
    }
}

pub struct ClientOptions {
    builder: Rc<RefCell<Option<ClientOptionsRust>>>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(ClientOptionsRust::default()))),
        }
    }
}

impl ClientOptions {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_with_builder(builder: ClientOptionsRust) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder))),
        }
    }

    pub fn with_primary_node(&mut self, node: &str) -> ClientOptions {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_primary_node(node)
            .unwrap();
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_primary_pow_node(&mut self, node: &str) -> ClientOptions {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_primary_pow_node(node)
            .unwrap();
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_node(&mut self, node: &str) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node(node).unwrap();
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_node_pool_urls(&mut self, node_pool_urls: Vec<String>) -> ClientOptions {
        let nodes_urls: Vec<&str> = node_pool_urls.iter().map(|x| &**x).collect();
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_node_pool_urls(&nodes_urls)
            .unwrap();
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_network(&mut self, network: String) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_network(network);
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_node_sync_interval(&mut self, node_sync_interval: Duration) -> ClientOptions {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_node_sync_interval(node_sync_interval);
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_node_sync_disabled(&mut self) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node_sync_disabled();
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_mqtt_disabled(&mut self) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_mqtt_disabled();
        ClientOptions::new_with_builder(new_builder)
    }

    /// Sets the MQTT broker options.
    pub fn with_mqtt_mqtt_broker_options(&mut self, options: BrokerOptions) -> ClientOptions {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_mqtt_mqtt_broker_options(options.builder.borrow_mut().take().unwrap());
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_local_pow(&mut self, local: bool) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_local_pow(local);
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_request_timeout(&mut self, timeout: Duration) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_request_timeout(timeout);
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn with_api_timeout(&mut self, api: Api, timeout: Duration) -> ClientOptions {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_api_timeout(api, timeout);
        ClientOptions::new_with_builder(new_builder)
    }

    pub fn build(&mut self) -> Result<ClientOptions> {
        Ok(ClientOptions {
            options: self.builder.borrow_mut().take().unwrap().build().unwrap(),
        })
    }
}

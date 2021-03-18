// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc, time::Duration};

use iota_wallet::client::{
    Api, BrokerOptions as BrokerOptionsRust, ClientOptions as ClientOptionsRust,
    ClientOptionsBuilder as ClientOptionsBuilderRust,
};

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
}

pub struct ClientOptions {
    options: ClientOptionsRust,
}

impl ClientOptions {
    pub fn to_inner(self) -> ClientOptionsRust {
        self.options
    }
}

pub struct ClientOptionsBuilder {
    builder: Rc<RefCell<Option<ClientOptionsBuilderRust>>>,
}

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(ClientOptionsBuilderRust::default()))),
        }
    }
}

impl ClientOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_with_builder(builder: ClientOptionsBuilderRust) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder))),
        }
    }

    pub fn with_node(&mut self, node: &str) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node(node).unwrap();
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_node_pool_urls(&mut self, node_pool_urls: Vec<String>) -> ClientOptionsBuilder {
        let nodes_urls: Vec<&str> = node_pool_urls.iter().map(|x| &**x).collect();
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_node_pool_urls(&nodes_urls)
            .unwrap();
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_network(&mut self, network: String) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_network(network);
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_node_sync_interval(&mut self, node_sync_interval: Duration) -> ClientOptionsBuilder {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_node_sync_interval(node_sync_interval);
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_node_sync_disabled(&mut self) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node_sync_disabled();
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    /// Sets the MQTT broker options.
    pub fn with_mqtt_mqtt_broker_options(&mut self, options: BrokerOptions) -> ClientOptionsBuilder {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_mqtt_mqtt_broker_options(options.builder.borrow_mut().take().unwrap());
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_local_pow(&mut self, local: bool) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_local_pow(local);
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_request_timeout(&mut self, timeout: Duration) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_request_timeout(timeout);
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn with_api_timeout(&mut self, api: Api, timeout: Duration) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_api_timeout(api, timeout);
        ClientOptionsBuilder::new_with_builder(new_builder)
    }

    pub fn build(&mut self) -> Result<ClientOptions> {
        Ok(ClientOptions {
            options: self.builder.borrow_mut().take().unwrap().build().unwrap(),
        })
    }
}

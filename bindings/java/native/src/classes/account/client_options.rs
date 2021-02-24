use std::{
    time::Duration,
    cell::RefCell,
    rc::Rc,
};

use iota_wallet::client::{
    ClientOptionsBuilder as ClientOptionsBuilderRust,
    ClientOptions as ClientOptionsRust,
    Api,
};

use crate::Result;

pub struct ClientOptions {
    options: ClientOptionsRust
}

impl ClientOptions {
    pub fn get_internal(self) -> ClientOptionsRust {
        // TODO: Find a way to not need clone
        self.options.clone()
    }
}

pub struct ClientOptionsBuilder {
    builder: Rc<RefCell<Option<ClientOptionsBuilderRust>>>
}

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(ClientOptionsBuilderRust::default())))
        }
    }
}

impl ClientOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_with_builder(builder: Rc<RefCell<Option<ClientOptionsBuilderRust>>>) -> Self {
        Self {
            builder: builder
        }
    }

    pub fn with_node(&mut self, node: &str) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node(node).unwrap();
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_node_pool_urls(&mut self, node_pool_urls: Vec<String>) -> ClientOptionsBuilder {
        let nodes_urls: Vec<&str> = node_pool_urls.iter().map(|x| &**x).collect();
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node_pool_urls(&nodes_urls).unwrap();
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_network(&mut self, network: String) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_network(network);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_node_sync_interval(&mut self, node_sync_interval: Duration) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node_sync_interval(node_sync_interval);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_node_sync_disabled(&mut self) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_node_sync_disabled();
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    /// Sets the MQTT broker options.
    /*pub fn with_mqtt_mqtt_broker_options(&mut self, options: BrokerOptions) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_mqtt_mqtt_broker_options(options);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }*/

    pub fn with_local_pow(&mut self, local: bool) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_local_pow(local);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_request_timeout(&mut self, timeout: Duration) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_request_timeout(timeout);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn with_api_timeout(&mut self, api: Api, timeout: Duration) -> ClientOptionsBuilder {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_api_timeout(api, timeout);
        ClientOptionsBuilder::new_with_builder(Rc::new(RefCell::new(Option::from(new_builder))))
    }

    pub fn build(&mut self) -> Result<ClientOptions> {
        Ok(ClientOptions {
            options: self.builder.borrow_mut().take().unwrap().build().unwrap()
        })
    }
}
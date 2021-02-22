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

pub struct ClientOptionsBuilder {
    builder: ClientOptionsBuilderRust
}

impl Default for ClientOptionsBuilder {
    fn default() -> Self {
        Self {
            builder: ClientOptionsBuilderRust::default()
        }
    }
}

impl ClientOptionsBuilder {
    /// Initialises a new instance of the builder.
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_nodes(&mut self, nodes: Vec<String>) {
        let nodes_urls: Vec<&str> = nodes.iter().map(|x| &**x).collect();
        self.builder.with_nodes(&nodes_urls);
    }

    /// Adds a node to the node list.
    pub fn with_node(&mut self, node: &str){
        self.builder.with_node(node);
    }

    /// Get node list from the node_pool_urls
    pub fn with_node_pool_urls(&mut self, node_pool_urls: Vec<String>) {
        let nodes_urls: Vec<&str> = node_pool_urls.iter().map(|x| &**x).collect();
        self.builder.with_node_pool_urls(&nodes_urls);
    }

    /// ClientOptions connected to the default Network pool.
    ///
    /// # Examples
    /// ```
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// let client_options = ClientOptionsBuilder::new().with_network("testnet2").build();
    /// ```
    pub fn with_network(&mut self, network: String) {
        self.builder.with_network(network);
    }

    /// Set the node sync interval
    pub fn with_node_sync_interval(&mut self, node_sync_interval: Duration) {
        self.builder.with_node_sync_interval(node_sync_interval);
    }

    /// Disables the node syncing process.
    /// Every node will be considered healthy and ready to use.
    pub fn with_node_sync_disabled(&mut self) {
        self.builder.with_node_sync_disabled();
    }

    /// Sets the MQTT broker options.
    /*pub fn with_mqtt_mqtt_broker_options(mut self, options: BrokerOptions) -> &mut Self {
        self.builder.with_mqtt_mqtt_broker_options(options);
        self
    }*/

    /// Sets whether the PoW should be done locally or remotely.
    pub fn with_local_pow(&mut self, local: bool) {
        self.builder.with_local_pow(local);
    }

    /// Sets the request timeout.
    pub fn with_request_timeout(&mut self, timeout: Duration) {
        self.builder.with_request_timeout(timeout);
    }

    /// Sets the request timeout for a specific API usage.
    pub fn with_api_timeout(&mut self, api: Api, timeout: Duration) {
        self.builder.with_api_timeout(api, timeout);
    }

    /// Builds the options.
    pub fn build(self) -> Result<ClientOptions> {
        Ok(ClientOptions {
            options: self.builder.build().unwrap()
        })
    }
}
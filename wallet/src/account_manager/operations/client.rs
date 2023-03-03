// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use iota_client::{
    node_manager::node::{Node, NodeAuth, NodeDto},
    NodeInfoWrapper, Url,
};

use crate::{
    account_manager::{builder::AccountManagerBuilder, AccountManager},
    ClientOptions,
};

impl AccountManager {
    /// Sets the client options for all accounts and sets the new bech32_hrp for the addresses.
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[set_client_options]");

        let mut client_options = self.client_options.write().await;
        *client_options = options.clone();
        drop(client_options);

        let new_client = options.clone().finish()?;

        #[allow(clippy::significant_drop_in_scrutinee)]
        for account in self.accounts.write().await.iter_mut() {
            account.update_account_with_new_client(new_client.clone()).await?;
        }

        #[cfg(feature = "storage")]
        {
            // Update account manager data with new client options
            let account_manager_builder = AccountManagerBuilder::from_account_manager(self)
                .await
                .with_client_options(options);

            self.storage_manager
                .lock()
                .await
                .save_account_manager_data(&account_manager_builder)
                .await?;
        }

        Ok(())
    }

    /// Get the used client options.
    pub async fn get_client_options(&self) -> ClientOptions {
        self.client_options.read().await.clone()
    }

    /// Get the node info.
    pub async fn get_node_info(&self) -> crate::Result<NodeInfoWrapper> {
        let accounts = self.accounts.read().await;

        // Try to get the Client from the first account and only build the Client if we have no account
        let node_info_wrapper = match &accounts.first() {
            Some(account) => account.client.get_info().await?,
            None => self.client_options.read().await.clone().finish()?.get_info().await?,
        };

        Ok(node_info_wrapper)
    }

    /// Update the authentication for a node.
    pub async fn update_node_auth(&self, url: Url, auth: Option<NodeAuth>) -> crate::Result<()> {
        log::debug!("[update_node_auth]");
        let mut client_options = self.client_options.write().await;

        if let Some(primary_node) = &client_options.node_manager_builder.primary_node {
            let (node_url, disabled) = match &primary_node {
                NodeDto::Url(node_url) => (node_url, false),
                NodeDto::Node(node) => (&node.url, node.disabled),
            };

            if node_url == &url {
                client_options.node_manager_builder.primary_node = Some(NodeDto::Node(Node {
                    url: url.clone(),
                    auth: auth.clone(),
                    disabled,
                }));
            }
        }

        if let Some(primary_pow_node) = &client_options.node_manager_builder.primary_pow_node {
            let (node_url, disabled) = match &primary_pow_node {
                NodeDto::Url(node_url) => (node_url, false),
                NodeDto::Node(node) => (&node.url, node.disabled),
            };

            if node_url == &url {
                client_options.node_manager_builder.primary_pow_node = Some(NodeDto::Node(Node {
                    url: url.clone(),
                    auth: auth.clone(),
                    disabled,
                }));
            }
        }

        if let Some(permanodes) = &client_options.node_manager_builder.permanodes {
            let mut new_permanodes = HashSet::new();
            for node in permanodes.iter() {
                let (node_url, disabled) = match &node {
                    NodeDto::Url(node_url) => (node_url, false),
                    NodeDto::Node(node) => (&node.url, node.disabled),
                };

                if node_url == &url {
                    new_permanodes.insert(NodeDto::Node(Node {
                        url: url.clone(),
                        auth: auth.clone(),
                        disabled,
                    }));
                } else {
                    new_permanodes.insert(node.clone());
                }
            }
            client_options.node_manager_builder.permanodes = Some(new_permanodes);
        }

        let mut new_nodes = HashSet::new();
        #[allow(clippy::significant_drop_in_scrutinee)]
        for node in client_options.node_manager_builder.nodes.iter() {
            let (node_url, disabled) = match &node {
                NodeDto::Url(node_url) => (node_url, false),
                NodeDto::Node(node) => (&node.url, node.disabled),
            };

            if node_url == &url {
                new_nodes.insert(NodeDto::Node(Node {
                    url: url.clone(),
                    auth: auth.clone(),
                    disabled,
                }));
            } else {
                new_nodes.insert(node.clone());
            }
        }
        client_options.node_manager_builder.nodes = new_nodes;

        let new_client_options = client_options.clone();
        // Need to drop client_options here to prevent a deadlock
        drop(client_options);

        #[cfg(feature = "storage")]
        {
            // Update account manager data with new client options
            let account_manager_builder = AccountManagerBuilder::from_account_manager(self)
                .await
                .with_client_options(new_client_options.clone());

            self.storage_manager
                .lock()
                .await
                .save_account_manager_data(&account_manager_builder)
                .await?;
        }

        let new_client = new_client_options.finish()?;

        #[allow(clippy::significant_drop_in_scrutinee)]
        for account in self.accounts.write().await.iter_mut() {
            account.update_account_with_new_client(new_client.clone()).await?;
        }

        Ok(())
    }
}

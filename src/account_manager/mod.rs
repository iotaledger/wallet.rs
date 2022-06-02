// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod builder;
pub(crate) mod operations;

use std::{
    collections::hash_map::Entry,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use iota_client::{secret::SecretManager, Client, NodeInfoWrapper};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use self::builder::AccountManagerBuilder;
#[cfg(feature = "storage")]
use self::builder::StorageOptions;
#[cfg(feature = "events")]
use crate::events::{
    types::{Event, WalletEventType},
    EventEmitter,
};
#[cfg(feature = "storage")]
use crate::storage::manager::StorageManagerHandle;
use crate::{
    account::{
        builder::AccountBuilder, handle::AccountHandle, operations::syncing::SyncOptions, types::AccountBalance,
    },
    ClientOptions,
};

/// The account manager, used to create and get accounts. One account manager can hold many accounts, but they should
/// all share the same secret_manager type with the same seed/mnemonic.
pub struct AccountManager {
    // should we use a hashmap instead of a vec like in wallet.rs?
    pub(crate) accounts: Arc<RwLock<Vec<AccountHandle>>>,
    // 0 = not running, 1 = running, 2 = stopping
    pub(crate) background_syncing_status: Arc<AtomicUsize>,
    pub(crate) client_options: Arc<RwLock<ClientOptions>>,
    pub(crate) secret_manager: Arc<RwLock<SecretManager>>,
    #[cfg(feature = "events")]
    pub(crate) event_emitter: Arc<Mutex<EventEmitter>>,
    #[cfg(feature = "storage")]
    pub(crate) storage_options: StorageOptions,
    #[cfg(feature = "storage")]
    pub(crate) storage_manager: StorageManagerHandle,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    /// Create a new account
    pub fn create_account(&self) -> AccountBuilder {
        log::debug!("creating account");
        AccountBuilder::new(
            self.accounts.clone(),
            self.client_options.clone(),
            self.secret_manager.clone(),
            #[cfg(feature = "events")]
            self.event_emitter.clone(),
            #[cfg(feature = "storage")]
            self.storage_manager.clone(),
        )
    }

    /// Get all accounts
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        Ok(self.accounts.read().await.clone())
    }

    /// Remove the latest account (account with the largest account index).
    pub async fn remove_latest_account(&mut self) -> crate::Result<()> {
        let mut accounts = self.accounts.write().await;

        let mut largest_account_index_opt = None;
        for account in accounts.iter() {
            let account_index = *account.read().await.index();
            if let Some(largest_account_index) = largest_account_index_opt {
                if account_index > largest_account_index {
                    largest_account_index_opt = Some(account_index);
                }
            } else {
                largest_account_index_opt = Some(account_index)
            }
        }

        if let Some(largest_account_index) = largest_account_index_opt {
            let mut i = 0;
            while i < accounts.len() {
                if let Some(account) = accounts.get(i) {
                    if *account.read().await.index() == largest_account_index {
                        let _ = accounts.remove(i);
                        break;
                    }
                }
                i = i + 1;
            }

            #[cfg(feature = "storage")]
            self.storage_manager
                .lock()
                .await
                .remove_account(largest_account_index)
                .await?;
        }

        Ok(())
    }

    /// Get the [SecretManager]
    pub fn get_secret_manager(&self) -> Arc<RwLock<SecretManager>> {
        self.secret_manager.clone()
    }

    /// Sets the client options for all accounts, syncs them and sets the new bech32_hrp
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[set_client_options]");
        let mut client_options = self.client_options.write().await;
        *client_options = options.clone();
        let new_client = options.clone().finish().await?;
        let mut accounts = self.accounts.write().await;
        for account in accounts.iter_mut() {
            account.update_account_with_new_client(new_client.clone()).await?;
        }
        #[cfg(feature = "storage")]
        {
            // Update account manager data with new client options
            let account_manager_builder = AccountManagerBuilder::new()
                .with_secret_manager_arc(self.secret_manager.clone())
                .with_storage_path(
                    &self
                        .storage_options
                        .storage_path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .expect("Can't convert os string"),
                )
                .with_client_options(options);

            self.storage_manager
                .lock()
                .await
                .save_account_manager_data(&account_manager_builder)
                .await?;
        }
        Ok(())
    }

    /// Get the used client options
    pub async fn get_client_options(&self) -> ClientOptions {
        self.client_options.read().await.clone()
    }

    /// Get the node info
    pub async fn get_node_info(&self) -> crate::Result<NodeInfoWrapper> {
        let accounts = self.accounts.read().await;

        // Try to get the Client from the first account and only build the Client if we have no account
        let node_info_wrapper = match &accounts.first() {
            Some(account) => account.client.get_info().await?,
            None => {
                self.client_options
                    .read()
                    .await
                    .clone()
                    .finish()
                    .await?
                    .get_info()
                    .await?
            }
        };

        Ok(node_info_wrapper)
    }

    /// Get the balance of all accounts added together
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        let mut balance = AccountBalance { ..Default::default() };
        let accounts = self.accounts.read().await;
        for account in accounts.iter() {
            let account_balance = account.balance().await?;
            balance.total += account_balance.total;
            balance.available += account_balance.available;
            // todo set other values
        }
        Ok(balance)
    }

    /// Sync all accounts
    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        let mut balance = AccountBalance { ..Default::default() };
        let accounts = self.accounts.read().await;
        for account in accounts.iter() {
            let account_balance = account.sync(options.clone()).await?;
            balance.total += account_balance.total;
            balance.available += account_balance.available;
            balance.required_storage_deposit += account_balance.required_storage_deposit;
            balance.nfts.extend(account_balance.nfts.into_iter());
            balance.aliases.extend(account_balance.aliases.into_iter());
            balance.foundries.extend(account_balance.foundries.into_iter());
            for (token_id, amount) in account_balance.native_tokens {
                match balance.native_tokens.entry(token_id) {
                    Entry::Vacant(e) => {
                        e.insert(amount);
                    }
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += amount;
                    }
                }
            }
        }
        Ok(balance)
    }

    /// Stop the background syncing of the accounts
    pub fn stop_background_syncing(&self) -> crate::Result<()> {
        log::debug!("[stop_background_syncing]");
        // send stop request
        self.background_syncing_status.store(2, Ordering::Relaxed);
        Ok(())
    }

    #[cfg(feature = "events")]
    /// Listen to wallet events, empty vec will listen to all events
    pub async fn listen<F>(&self, events: Vec<WalletEventType>, handler: F)
    where
        F: Fn(&Event) + 'static + Clone + Send + Sync,
    {
        let mut emitter = self.event_emitter.lock().await;
        emitter.on(events, handler);
    }

    /// Generates a new random mnemonic.
    pub fn generate_mnemonic(&self) -> crate::Result<String> {
        Ok(Client::generate_mnemonic()?)
    }

    /// Verify that a &str is a valid mnemonic.
    pub fn verify_mnemonic(&self, mnemonic: &str) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::keys::bip39::wordlist::verify(mnemonic, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;
        Ok(())
    }

    #[cfg(feature = "events")]
    #[cfg(debug_assertions)]
    /// Helper function to test events. Emits a provided event with account index 0.
    pub async fn emit_test_event(&self, event: crate::events::types::WalletEvent) -> crate::Result<()> {
        self.event_emitter.lock().await.emit(0, event);
        Ok(())
    }

    #[cfg(feature = "storage")]
    pub async fn delete_storage(&self) -> crate::Result<()> {
        std::fs::remove_dir_all(self.storage_options.storage_path.clone())?;
        Ok(())
    }
}

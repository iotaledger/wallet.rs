// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod builder;
pub(crate) mod operations;

use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use iota_client::{secret::SecretManager, Client};
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
        builder::AccountBuilder,
        handle::AccountHandle,
        operations::{balance::add_balances, syncing::SyncOptions},
        types::AccountBalance,
    },
    ClientOptions,
};

/// The account manager, used to create and get accounts. One account manager can hold many accounts, but they should
/// all share the same secret_manager type with the same seed/mnemonic.
#[derive(Debug)]
pub struct AccountManager {
    // should we use a hashmap instead of a vec like in wallet.rs?
    pub(crate) accounts: Arc<RwLock<Vec<AccountHandle>>>,
    // 0 = not running, 1 = running, 2 = stopping
    pub(crate) background_syncing_status: Arc<AtomicUsize>,
    pub(crate) client_options: Arc<RwLock<ClientOptions>>,
    pub(crate) coin_type: Arc<AtomicU32>,
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
            self.coin_type.load(Ordering::Relaxed),
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

    /// Removes the latest account (account with the largest account index).
    pub async fn remove_latest_account(&self) -> crate::Result<()> {
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
            for i in 0..accounts.len() {
                if let Some(account) = accounts.get(i) {
                    if *account.read().await.index() == largest_account_index {
                        let _ = accounts.remove(i);

                        #[cfg(feature = "storage")]
                        self.storage_manager
                            .lock()
                            .await
                            .remove_account(largest_account_index)
                            .await?;

                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the [SecretManager]
    pub fn get_secret_manager(&self) -> Arc<RwLock<SecretManager>> {
        self.secret_manager.clone()
    }

    /// Get the balance of all accounts added together
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        let accounts = self.accounts.read().await;

        let mut balances = Vec::new();
        for account in accounts.iter() {
            let account_balance = account.balance().await?;
            balances.push(account_balance);
        }

        let balance = add_balances(balances)?;

        Ok(balance)
    }

    /// Sync all accounts
    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        let accounts = self.accounts.read().await;

        let mut balances = Vec::new();
        for account in accounts.iter() {
            let account_balance = account.sync(options.clone()).await?;
            balances.push(account_balance);
        }

        let balance = add_balances(balances)?;

        Ok(balance)
    }

    /// Listen to wallet events, empty vec will listen to all events
    #[cfg(feature = "events")]
    #[cfg_attr(docsrs, doc(cfg(feature = "events")))]
    pub async fn listen<F>(&self, events: Vec<WalletEventType>, handler: F)
    where
        F: Fn(&Event) + 'static + Clone + Send + Sync,
    {
        let mut emitter = self.event_emitter.lock().await;
        emitter.on(events, handler);
    }

    /// Remove wallet event listeners, empty vec will remove all listeners
    #[cfg(feature = "events")]
    #[cfg_attr(docsrs, doc(cfg(feature = "events")))]
    pub async fn clear_listeners(&self, events: Vec<WalletEventType>) {
        let mut emitter = self.event_emitter.lock().await;
        emitter.clear(events);
    }

    /// Generates a new random mnemonic.
    pub fn generate_mnemonic(&self) -> crate::Result<String> {
        Ok(Client::generate_mnemonic()?)
    }

    /// Verify that a &str is a valid mnemonic.
    pub fn verify_mnemonic(&self, mnemonic: &str) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::keys::bip39::wordlist::verify(mnemonic, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{e:?}")))?;
        Ok(())
    }

    /// Helper function to test events. Emits a provided event with account index 0.
    #[cfg(feature = "events")]
    #[cfg_attr(docsrs, doc(cfg(feature = "events")))]
    pub async fn emit_test_event(&self, event: crate::events::types::WalletEvent) -> crate::Result<()> {
        self.event_emitter.lock().await.emit(0, event);
        Ok(())
    }
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        log::debug!("drop AccountManager");
    }
}

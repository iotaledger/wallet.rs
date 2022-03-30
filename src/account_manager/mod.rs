// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod builder;
pub(crate) mod operations;

#[cfg(feature = "storage")]
use crate::account_manager::builder::StorageOptions;
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
use builder::AccountManagerBuilder;

use iota_client::{signing::SignerHandle, Client};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use std::path::Path;
use std::{
    collections::hash_map::Entry,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// The account manager, used to create and get accounts. One account manager can hold many accounts, but they should
/// all share the same signer type with the same seed/mnemonic.
pub struct AccountManager {
    // should we use a hashmap instead of a vec like in wallet.rs?
    pub(crate) accounts: Arc<RwLock<Vec<AccountHandle>>>,
    // 0 = not running, 1 = running, 2 = stopping
    pub(crate) background_syncing_status: Arc<AtomicUsize>,
    pub(crate) client_options: Arc<RwLock<ClientOptions>>,
    pub(crate) signer: SignerHandle,
    #[cfg(feature = "events")]
    pub(crate) event_emitter: Arc<Mutex<EventEmitter>>,
    #[cfg(feature = "storage")]
    pub(crate) storage_options: StorageOptions,
    #[cfg(feature = "storage")]
    pub(crate) storage_manager: StorageManagerHandle,
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder(signer: SignerHandle) -> AccountManagerBuilder {
        AccountManagerBuilder::new(signer)
    }

    /// Create a new account
    pub fn create_account(&self) -> AccountBuilder {
        log::debug!("creating account");
        AccountBuilder::new(
            self.accounts.clone(),
            self.client_options.clone(),
            self.signer.clone(),
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

    /// Get the [SignerHandle]
    pub fn get_signer(&self) -> SignerHandle {
        self.signer.clone()
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
            let account_manager_builder = AccountManagerBuilder::new(self.signer.clone())
                .with_storage_folder(
                    &self
                        .storage_options
                        .storage_folder
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

    // storage feature
    #[cfg(feature = "storage")]
    pub async fn backup<P: AsRef<Path>>(&self, destination: P, stronghold_password: String) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn restore_backup<S: AsRef<Path>>(&self, source: S, stronghold_password: String) -> crate::Result<()> {
        Ok(())
    }
    #[cfg(feature = "storage")]
    pub async fn delete_storage(&self) -> crate::Result<()> {
        Ok(())
    }
}

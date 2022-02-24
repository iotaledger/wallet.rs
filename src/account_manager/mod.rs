// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod builder;
pub(crate) mod operations;

#[cfg(feature = "events")]
use crate::events::{
    types::{Event, WalletEventType},
    EventEmitter,
};
use crate::{
    account::{
        builder::AccountBuilder,
        handle::AccountHandle,
        operations::syncing::SyncOptions,
        types::{AccountBalance, AccountIdentifier},
    },
    ClientOptions,
};
use builder::AccountManagerBuilder;
use operations::{get_account, recover_accounts, start_background_syncing, verify_integrity};

use iota_client::{signing::SignerHandle, Client};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
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
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    /// Create a new account
    pub fn create_account(&self) -> AccountBuilder {
        log::debug!("creating account");
        #[cfg(not(feature = "events"))]
        return AccountBuilder::new(self.accounts.clone(), self.signer_type.clone());
        #[cfg(feature = "events")]
        AccountBuilder::new(
            self.accounts.clone(),
            self.client_options.clone(),
            self.signer.clone(),
            self.event_emitter.clone(),
        )
    }
    /// Get an account with an AccountIdentifier
    pub async fn get_account<I: Into<AccountIdentifier>>(&self, identifier: I) -> crate::Result<AccountHandle> {
        get_account(self, identifier).await
    }
    /// Get all accounts
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        Ok(self.accounts.read().await.clone())
    }

    // do want a function to delete an account? If so we have to change the account creation logic, otherwise multiple
    // accounts could get the same index /// Delete an account
    // pub async fn delete_account(&self, identifier: AccountIdentifier) -> crate::Result<()> {
    // Ok(())
    // }

    /// Find accounts with balances
    /// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
    /// has balance, the counter is reset
    /// `account_gap_limit` defines how many accounts without balance will be
    /// checked, if an account has balance, the counter is reset
    pub async fn recover_accounts(
        &self,
        address_gap_limit: u32,
        account_gap_limit: u32,
    ) -> crate::Result<Vec<AccountHandle>> {
        recover_accounts(self, address_gap_limit, account_gap_limit).await
    }

    /// Sets the client options for all accounts, syncs them and sets the new bech32_hrp
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        log::debug!("[set_client_options]");
        let mut client_options = self.client_options.write().await;
        *client_options = options.clone();
        let new_client = options.finish().await?;
        let mut accounts = self.accounts.write().await;
        for account in accounts.iter_mut() {
            account.update_account_with_new_client(new_client.clone()).await?;
        }
        Ok(())
    }

    /// Get the balance of all accounts added together
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        let mut balance = AccountBalance {
            total: 0,
            available: 0,
            // todo set other values
            ..Default::default()
        };
        let accounts = self.accounts.read().await;
        for account in accounts.iter() {
            let account_balance = account.balance().await?;
            balance.total += account_balance.total;
            balance.available += account_balance.available;
        }
        Ok(balance)
    }

    /// Start the background syncing process for all accounts, default interval is 7 seconds
    pub async fn start_background_syncing(
        &self,
        options: Option<SyncOptions>,
        interval: Option<Duration>,
    ) -> crate::Result<()> {
        start_background_syncing(self, options, interval).await
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

    /// Checks if there is no missing account for example indexes [0, 1, 3] should panic (for now, later return error,
    /// automatically fix?) Also checks for each account if there is a gap in an address list and no address is
    /// duplicated
    pub async fn verify_integrity(&self) -> crate::Result<()> {
        verify_integrity(self).await
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

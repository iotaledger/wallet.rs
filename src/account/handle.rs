// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{ops::Deref, sync::Arc};

use iota_client::{secret::SecretManagerType, Client};
use tokio::sync::{Mutex, RwLock};

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::StorageManagerHandle;
use crate::{
    account::{
        operations::syncing::SyncOptions,
        types::{
            address::{AccountAddress, AddressWithUnspentOutputs},
            OutputData, Transaction,
        },
        Account,
    },
    Result,
};

/// A thread guard over an account, so we can lock the account during operations.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
    pub(crate) client: Client,
    pub(crate) secret_manager: Arc<RwLock<SecretManagerType>>,
    // mutex to prevent multiple sync calls at the same or almost the same time, the u128 is a timestamp
    // if the last synced time was < `MIN_SYNC_INTERVAL` second ago, we don't sync, but only calculate the balance
    // again, because sending transactions can change that
    pub(crate) last_synced: Arc<Mutex<u128>>,
    #[cfg(feature = "events")]
    pub(crate) event_emitter: Arc<Mutex<EventEmitter>>,
    #[cfg(feature = "storage")]
    pub(crate) storage_manager: StorageManagerHandle,
}

impl AccountHandle {
    /// Create a new AccountHandle with an Account
    pub(crate) fn new(
        account: Account,
        client: Client,
        secret_manager: Arc<RwLock<SecretManagerType>>,
        #[cfg(feature = "events")] event_emitter: Arc<Mutex<EventEmitter>>,
        #[cfg(feature = "storage")] storage_manager: StorageManagerHandle,
    ) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
            client,
            secret_manager,
            last_synced: Default::default(),
            #[cfg(feature = "events")]
            event_emitter,
            #[cfg(feature = "storage")]
            storage_manager,
        }
    }

    pub async fn alias(&self) -> String {
        self.read().await.alias.clone()
    }

    /// Returns all addresses of the account
    pub async fn list_addresses(&self) -> Result<Vec<AccountAddress>> {
        let account = self.read().await;
        let mut all_addresses = account.public_addresses().clone();
        all_addresses.extend(account.internal_addresses().clone());
        Ok(all_addresses.to_vec())
    }

    /// Returns only addresses of the account with balance
    pub async fn list_addresses_with_unspent_outputs(&self) -> Result<Vec<AddressWithUnspentOutputs>> {
        let account = self.read().await;
        Ok(account.addresses_with_unspent_outputs().to_vec())
    }

    /// Returns all outputs of the account
    pub async fn list_outputs(&self) -> Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all unspent outputs of the account
    pub async fn list_unspent_outputs(&self) -> Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.unspent_outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all transaction of the account
    pub async fn list_transactions(&self) -> Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction in account.transactions.values() {
            transactions.push(transaction.clone());
        }
        Ok(transactions)
    }

    /// Returns all pending transaction of the account
    pub async fn list_pending_transactions(&self) -> Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction_id in &account.pending_transactions {
            if let Some(transaction) = account.transactions.get(transaction_id) {
                transactions.push(transaction.clone());
            }
        }
        Ok(transactions)
    }

    #[cfg(feature = "storage")]
    /// Save the account to the database, accepts the updated_account as option so we don't need to drop it before
    /// saving
    pub(crate) async fn save(&self, updated_account: Option<&Account>) -> Result<()> {
        log::debug!("[save] saving account to database");
        match updated_account {
            Some(account) => self.storage_manager.lock().await.save_account(account).await,
            None => {
                let account = self.read().await;
                self.storage_manager.lock().await.save_account(&account).await
            }
        }
    }

    // Set the alias for the account
    pub async fn set_alias(&self, alias: &str) -> Result<()> {
        let mut account = self.write().await;
        account.alias = alias.to_string();
        #[cfg(feature = "storage")]
        self.save(Some(&account)).await?;
        Ok(())
    }

    // Should only be called from the AccountManager so all accounts are on the same state
    pub(crate) async fn update_account_with_new_client(&mut self, client: Client) -> Result<()> {
        self.client = client;
        let bech32_hrp = self.client.get_bech32_hrp().await?;
        log::debug!("[UPDATE ACCOUNT WITH NEW CLIENT] new bech32_hrp: {}", bech32_hrp);
        let mut account = self.account.write().await;
        for address in &mut account.addresses_with_unspent_outputs {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.public_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.internal_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        // Drop account before syncing because we locked it
        drop(account);
        // after we set the new client options we should sync the account because the network could have changed
        // we sync with all addresses, because otherwise the balance wouldn't get updated if an address doesn't has
        // balance also in the new network
        self.sync(Some(SyncOptions {
            force_syncing: true,
            ..Default::default()
        }))
        .await?;
        Ok(())
    }
}

// impl Deref so we can use `account_handle.read()` instead of `account_handle.account.read()`
impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        self.account.deref()
    }
}

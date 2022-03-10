// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    operations::syncing::SyncOptions,
    types::{
        address::{AccountAddress, AddressWithBalance},
        OutputData, Transaction,
    },
    Account,
};
#[cfg(feature = "events")]
use crate::events::EventEmitter;

use iota_client::{signing::SignerHandle, Client};

use tokio::sync::{Mutex, RwLock};

use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Deref,
    sync::Arc,
};

/// A thread guard over an account, so we can lock the account during operations.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
    pub(crate) client: Client,
    pub(crate) signer: SignerHandle,
    // mutex to prevent multiple sync calls at the same or almost the same time, the u128 is a timestamp
    // if the last synced time was < `MIN_SYNC_INTERVAL` second ago, we don't sync, but only calculate the balance
    // again, because sending transactions can change that
    pub(crate) last_synced: Arc<Mutex<u128>>,
    #[cfg(feature = "events")]
    pub(crate) event_emitter: Arc<Mutex<EventEmitter>>,
}

impl AccountHandle {
    /// Create a new AccountHandle with an Account
    #[cfg(not(feature = "events"))]
    pub(crate) fn new(account: Account, client: Client, signer: SignerHandle) -> Self {
        Self {
            signer,
            client,
            account: Arc::new(RwLock::new(account)),
            last_synced: Default::default(),
        }
    }
    #[cfg(feature = "events")]
    pub(crate) fn new(
        account: Account,
        client: Client,
        signer: SignerHandle,
        event_emitter: Arc<Mutex<EventEmitter>>,
    ) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
            client,
            signer,
            last_synced: Default::default(),
            event_emitter,
        }
    }

    /// Returns all addresses of the account
    pub async fn list_addresses(&self) -> crate::Result<Vec<AccountAddress>> {
        let account = self.read().await;
        let mut all_addresses = account.public_addresses().clone();
        all_addresses.extend(account.internal_addresses().clone());
        Ok(all_addresses.to_vec())
    }

    /// Returns only addresses of the account with balance
    pub async fn list_addresses_with_balance(&self) -> crate::Result<Vec<AddressWithBalance>> {
        let account = self.read().await;
        Ok(account.addresses_with_balance().to_vec())
    }

    /// Returns all outputs of the account
    pub async fn list_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all unspent outputs of the account
    pub async fn list_unspent_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.unspent_outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all transaction of the account
    pub async fn list_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction in account.transactions.values() {
            transactions.push(transaction.clone());
        }
        Ok(transactions)
    }

    /// Returns all pending transaction of the account
    pub async fn list_pending_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction_id in &account.pending_transactions {
            if let Some(transaction) = account.transactions.get(transaction_id) {
                transactions.push(transaction.clone());
            }
        }
        Ok(transactions)
    }

    // Should only be called from the AccountManager so all accounts are on the same state
    pub(crate) async fn update_account_with_new_client(&mut self, client: Client) -> crate::Result<()> {
        self.client = client;
        let bech32_hrp = self.client.get_bech32_hrp().await?;
        log::debug!("[UPDATE ACCOUNT WITH NEW CLIENT] new bech32_hrp: {}", bech32_hrp);
        let mut account = self.account.write().await;
        for address in &mut account.addresses_with_balance {
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
            sync_all_addresses: true,
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

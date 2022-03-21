// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "ledger-nano")]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::StorageManagerHandle;
use crate::{
    account::{constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle, Account, AccountOptions},
    ClientOptions, Error,
};

#[cfg(feature = "ledger-nano")]
use iota_client::signing::SignerType;
use iota_client::{constants::IOTA_COIN_TYPE, signing::SignerHandle};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// The AccountBuilder
pub struct AccountBuilder {
    client_options: Arc<RwLock<ClientOptions>>,
    alias: Option<String>,
    signer: SignerHandle,
    accounts: Arc<RwLock<Vec<AccountHandle>>>,
    #[cfg(feature = "events")]
    event_emitter: Arc<Mutex<EventEmitter>>,
    #[cfg(feature = "storage")]
    storage_manager: StorageManagerHandle,
}

impl AccountBuilder {
    /// Create an IOTA client builder
    pub fn new(
        accounts: Arc<RwLock<Vec<AccountHandle>>>,
        client_options: Arc<RwLock<ClientOptions>>,
        signer: SignerHandle,
        #[cfg(feature = "events")] event_emitter: Arc<Mutex<EventEmitter>>,
        #[cfg(feature = "storage")] storage_manager: StorageManagerHandle,
    ) -> Self {
        Self {
            client_options,
            alias: None,
            signer,
            accounts,
            #[cfg(feature = "events")]
            event_emitter,
            #[cfg(feature = "storage")]
            storage_manager,
        }
    }

    /// Set the alias
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias.replace(alias);
        self
    }

    // Build the Account
    pub async fn finish(&self) -> crate::Result<AccountHandle> {
        let mut accounts = self.accounts.write().await;
        let account_index = accounts.len() as u32;
        // If no alias is provided, the account index will be set as alias
        let account_alias = self.alias.clone().unwrap_or_else(|| account_index.to_string());

        // Check that the alias isn't already used for another account
        for account_handle in accounts.iter() {
            if account_handle.read().await.alias().to_lowercase() == account_alias.to_lowercase() {
                return Err(Error::AccountAliasAlreadyExists);
            }
        }

        let consolidation_threshold = match self.signer.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano | SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        };
        let account = Account {
            index: account_index,
            coin_type: IOTA_COIN_TYPE,
            alias: account_alias,
            public_addresses: Vec::new(),
            internal_addresses: Vec::new(),
            addresses_with_balance: Vec::new(),
            outputs: HashMap::new(),
            locked_outputs: HashSet::new(),
            unspent_outputs: HashMap::new(),
            transactions: HashMap::new(),
            pending_transactions: HashSet::new(),
            // sync interval, output consolidation
            account_options: AccountOptions {
                output_consolidation_threshold: consolidation_threshold,
                automatic_output_consolidation: true,
            },
        };
        let account_handle = AccountHandle::new(
            account,
            self.client_options.read().await.clone().finish().await?,
            self.signer.clone(),
            #[cfg(feature = "events")]
            self.event_emitter.clone(),
            #[cfg(feature = "storage")]
            self.storage_manager.clone(),
        );
        #[cfg(feature = "storage")]
        account_handle.save(None).await?;
        accounts.push(account_handle.clone());
        Ok(account_handle)
    }
}

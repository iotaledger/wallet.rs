// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
#[cfg(feature = "events")]
use crate::events::EventEmitter;
use crate::{
    account::{constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle, Account, AccountOptions},
    ClientOptions,
};

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
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
}

impl AccountBuilder {
    #[cfg(not(feature = "events"))]
    /// Create an IOTA client builder
    pub fn new(
        accounts: Arc<RwLock<Vec<AccountHandle>>>,
        client: Arc<RwLock<ClientOptions>>,
        signer: SignerHandle,
    ) -> Self {
        Self {
            client,
            alias: None,
            signer,
            accounts,
        }
    }

    #[cfg(feature = "events")]
    /// Create an IOTA client builder
    pub fn new(
        accounts: Arc<RwLock<Vec<AccountHandle>>>,
        client_options: Arc<RwLock<ClientOptions>>,
        signer: SignerHandle,
        event_emitter: Arc<Mutex<EventEmitter>>,
    ) -> Self {
        Self {
            client_options,
            alias: None,
            signer,
            accounts,
            event_emitter,
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
        let index = accounts.len() as u32;
        let consolidation_threshold = match self.signer.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        };
        let account = Account {
            index,
            coin_type: IOTA_COIN_TYPE,
            alias: self.alias.clone().unwrap_or_else(|| index.to_string()),
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
        #[cfg(feature = "storage")]
        log::debug!("[TRANSFER] storing account {}", account.index());
        crate::storage::manager::get()
            .await?
            .lock()
            .await
            .save_account(&account)
            .await?;
        #[cfg(not(feature = "events"))]
        let account_handle = AccountHandle::new(account);
        #[cfg(feature = "events")]
        let account_handle = AccountHandle::new(
            account,
            self.client_options.read().await.clone().finish().await?,
            self.signer.clone(),
            self.event_emitter.clone(),
        );
        accounts.push(account_handle.clone());
        Ok(account_handle)
    }
}

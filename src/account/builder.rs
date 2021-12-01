// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
#[cfg(feature = "events")]
use crate::events::EventEmitter;
use crate::{
    account::{constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle, Account, AccountOptions},
    client::options::ClientOptions,
    signing::SignerType,
};

#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// The AccountBuilder
pub struct AccountBuilder {
    client_options: Option<ClientOptions>,
    alias: Option<String>,
    signer_type: SignerType,
    accounts: Arc<RwLock<Vec<AccountHandle>>>,
    #[cfg(feature = "events")]
    event_emitter: Arc<Mutex<EventEmitter>>,
}

impl AccountBuilder {
    #[cfg(not(feature = "events"))]
    /// Create an IOTA client builder
    pub fn new(accounts: Arc<RwLock<Vec<AccountHandle>>>, signer_type: SignerType) -> Self {
        Self {
            client_options: None,
            alias: None,
            signer_type,
            accounts,
        }
    }

    #[cfg(feature = "events")]
    /// Create an IOTA client builder
    pub fn new(
        accounts: Arc<RwLock<Vec<AccountHandle>>>,
        signer_type: SignerType,
        event_emitter: Arc<Mutex<EventEmitter>>,
    ) -> Self {
        Self {
            client_options: None,
            alias: None,
            signer_type,
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
        let index = accounts.len();
        let consolidation_threshold = match self.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        };
        let account = Account {
            id: index.to_string(),
            index,
            alias: self.alias.clone().unwrap_or_else(|| index.to_string()),
            signer_type: self.signer_type.clone(),
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
        #[cfg(not(feature = "events"))]
        let account_handle = AccountHandle::new(account);
        #[cfg(feature = "events")]
        let account_handle = AccountHandle::new(account, self.event_emitter.clone());
        accounts.push(account_handle.clone());
        Ok(account_handle)
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[cfg(feature = "ledger-nano")]
use iota_client::signing::SignerType;
use iota_client::{
    bee_message::address::Address,
    constants::IOTA_COIN_TYPE,
    signing::{GenerateAddressMetadata, Network, SignerHandle},
};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "ledger-nano")]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::StorageManagerHandle;
use crate::{
    account::{
        constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        handle::AccountHandle,
        types::{address::AddressWrapper, AccountAddress},
        Account, AccountOptions,
    },
    ClientOptions, Error,
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

    /// Build the Account and add it to the accounts from AccountManager
    /// Also generates the first address of the account and if it's not the first account, the address for the first
    /// account will also be generated and compared, so no accounts get generated with different seeds
    pub async fn finish(&self) -> crate::Result<AccountHandle> {
        let mut accounts = self.accounts.write().await;
        let account_index = accounts.len() as u32;
        // If no alias is provided, the account index will be set as alias
        let account_alias = self.alias.clone().unwrap_or_else(|| account_index.to_string());
        log::debug!(
            "[ACCOUNT BUILDER] creating new account {} with index {}",
            account_alias,
            account_index
        );

        // Check that the alias isn't already used for another account
        for account_handle in accounts.iter() {
            if account_handle.read().await.alias().to_lowercase() == account_alias.to_lowercase() {
                return Err(Error::AccountAliasAlreadyExists);
            }
        }

        let mut bech32_hrp = None;
        if let Some(first_account) = accounts.first() {
            // Generate the first address of the first account and compare it to the stored address from the first
            // account to prevent having multiple accounts created with different seeds
            let first_account_public_address = get_first_public_address(&self.signer, IOTA_COIN_TYPE, 0).await?;
            let first_account_addresses = first_account.list_addresses().await?;

            if first_account_public_address
                != first_account_addresses
                    .first()
                    .ok_or(Error::FailedToGetRemainder)?
                    .address
                    .inner
            {
                return Err(Error::InvalidMnemonic(
                    "First account address used another seed".to_string(),
                ));
            }

            // Get bech32_hrp from address
            if let Some(address) = first_account_addresses.first() {
                bech32_hrp = Some(address.address.bech32_hrp.clone());
            }
        }

        let client = self.client_options.read().await.clone().finish().await?;
        // get bech32_hrp
        let bech32_hrp = {
            match bech32_hrp {
                Some(bech32_hrp) => bech32_hrp,
                // Only when we create a new account we don't have the first address and need to get the information
                // from the client Doesn't work for offline creating, should we use the network from the
                // GenerateAddressMetadata instead to use `iota` or `atoi`?
                None => {
                    let bech32_hrp = client.get_bech32_hrp().await.unwrap_or_else(|_| "iota".to_string());
                    bech32_hrp
                }
            }
        };

        let first_public_address = get_first_public_address(&self.signer, IOTA_COIN_TYPE, account_index).await?;

        let first_public_account_address = AccountAddress {
            address: AddressWrapper::new(first_public_address, bech32_hrp),
            key_index: 0,
            internal: false,
            used: false,
        };

        let consolidation_threshold = match self.signer.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano | SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
            _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
        };
        let account = Account {
            index: account_index,
            coin_type: IOTA_COIN_TYPE,
            alias: account_alias,
            public_addresses: vec![first_public_account_address],
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
            client,
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

/// Generate the first public address of an account
pub(crate) async fn get_first_public_address(
    signer: &SignerHandle,
    coin_type: u32,
    account_index: u32,
) -> crate::Result<Address> {
    Ok(signer
        .lock()
        .await
        .generate_addresses(
            coin_type,
            account_index,
            0..1,
            false,
            GenerateAddressMetadata {
                network: Network::Testnet,
                syncing: true,
            },
        )
        .await?[0])
}

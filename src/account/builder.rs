// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use iota_client::{
    block::address::Address,
    constants::SHIMMER_TESTNET_BECH32_HRP,
    secret::{SecretManage, SecretManager},
};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::StorageManagerHandle;
use crate::{
    account::{
        handle::AccountHandle,
        types::{address::AddressWrapper, AccountAddress},
        Account,
    },
    ClientOptions, Error,
};

/// The AccountBuilder
pub struct AccountBuilder {
    addresses: Option<Vec<AccountAddress>>,
    alias: Option<String>,
    client_options: Arc<RwLock<ClientOptions>>,
    coin_type: u32,
    secret_manager: Arc<RwLock<SecretManager>>,
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
        coin_type: u32,
        secret_manager: Arc<RwLock<SecretManager>>,
        #[cfg(feature = "events")] event_emitter: Arc<Mutex<EventEmitter>>,
        #[cfg(feature = "storage")] storage_manager: StorageManagerHandle,
    ) -> Self {
        Self {
            addresses: None,
            alias: None,
            client_options,
            coin_type,
            secret_manager,
            accounts,
            #[cfg(feature = "events")]
            event_emitter,
            #[cfg(feature = "storage")]
            storage_manager,
        }
    }

    /// Set the addresses, should only be used for accounts with an offline counterpart account from which the addresses
    /// were exported
    pub fn with_addresses(mut self, addresses: Vec<AccountAddress>) -> Self {
        self.addresses.replace(addresses);
        self
    }

    /// Set the alias
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias.replace(alias);
        self
    }

    /// Build the Account and add it to the accounts from AccountManager
    /// Also generates the first address of the account and if it's not the first account, the address for the first
    /// account will also be generated and compared, so no accounts get generated with different seeds
    pub async fn finish(&mut self) -> crate::Result<AccountHandle> {
        let mut accounts = self.accounts.write().await;
        let account_index = accounts.len() as u32;
        // If no alias is provided, the account index will be set as alias
        let account_alias = self.alias.clone().unwrap_or_else(|| account_index.to_string());
        log::debug!(
            "[ACCOUNT BUILDER] creating new account {} with index {}",
            account_alias,
            account_index
        );

        // Check that the alias isn't already used for another account and that the coin type is the same for new and
        // existing accounts
        for account_handle in accounts.iter() {
            let account = account_handle.read().await;
            let existing_coin_type = account.coin_type;
            if existing_coin_type != self.coin_type {
                return Err(Error::InvalidCoinType(self.coin_type, existing_coin_type));
            }
            if account.alias().to_lowercase() == account_alias.to_lowercase() {
                return Err(Error::AccountAliasAlreadyExists(account_alias));
            }
        }

        let client = self.client_options.read().await.clone().finish()?;

        // If addresses are provided we will use them directly without the additional checks, because then we assume
        // that it's for offline signing and the secretManager can't be used
        let addresses = match &self.addresses {
            Some(addresses) => addresses.clone(),
            None => {
                let mut bech32_hrp = None;
                if let Some(first_account) = accounts.first() {
                    let first_account_coin_type = *first_account.read().await.coin_type();
                    // Generate the first address of the first account and compare it to the stored address from the
                    // first account to prevent having multiple accounts created with different
                    // seeds
                    let first_account_public_address =
                        get_first_public_address(&self.secret_manager, first_account_coin_type, 0).await?;
                    let first_account_addresses = first_account.public_addresses().await;

                    if first_account_public_address
                        != first_account_addresses
                            .first()
                            .ok_or(Error::FailedToGetRemainder)?
                            .address
                            .inner
                    {
                        return Err(Error::InvalidMnemonic(
                            "first account address used another seed".to_string(),
                        ));
                    }

                    // Get bech32_hrp from address
                    if let Some(address) = first_account_addresses.first() {
                        bech32_hrp = Some(address.address.bech32_hrp.clone());
                    }
                }

                // get bech32_hrp
                let bech32_hrp = {
                    match bech32_hrp {
                        Some(bech32_hrp) => bech32_hrp,
                        // Only when we create a new account we don't have the first address and need to get the
                        // information from the client Doesn't work for offline creating, should
                        // we use the network from the GenerateAddressOptions instead to use
                        // `iota` or `atoi`?
                        None => client
                            .get_bech32_hrp()
                            .unwrap_or_else(|_| SHIMMER_TESTNET_BECH32_HRP.to_string()),
                    }
                };

                let first_public_address =
                    get_first_public_address(&self.secret_manager, self.coin_type, account_index).await?;

                let first_public_account_address = AccountAddress {
                    address: AddressWrapper::new(first_public_address, bech32_hrp),
                    key_index: 0,
                    internal: false,
                    used: false,
                };

                vec![first_public_account_address]
            }
        };

        let account = Account {
            index: account_index,
            coin_type: self.coin_type,
            alias: account_alias,
            public_addresses: addresses,
            internal_addresses: Vec::new(),
            addresses_with_unspent_outputs: Vec::new(),
            outputs: HashMap::new(),
            locked_outputs: HashSet::new(),
            unspent_outputs: HashMap::new(),
            transactions: HashMap::new(),
            pending_transactions: HashSet::new(),
            incoming_transactions: HashMap::new(),
        };

        let account_handle = AccountHandle::new(
            account,
            client,
            self.secret_manager.clone(),
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
    secret_manager: &Arc<RwLock<SecretManager>>,
    coin_type: u32,
    account_index: u32,
) -> crate::Result<Address> {
    Ok(secret_manager
        .read()
        .await
        .generate_addresses(coin_type, account_index, 0..1, false, None)
        .await?[0])
}

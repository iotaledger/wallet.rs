// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::{
    atomic::{AtomicU32, AtomicUsize},
    Arc,
};
#[cfg(feature = "storage")]
use std::{path::PathBuf, sync::atomic::Ordering};

use iota_client::secret::SecretManager;
use serde::{Deserialize, Serialize};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(all(feature = "storage", not(feature = "rocksdb")))]
use crate::storage::adapter::memory::Memory;
#[cfg(feature = "storage")]
use crate::{
    account::handle::AccountHandle,
    storage::{constants::default_storage_path, manager::ManagerStorage},
};
use crate::{account_manager::AccountManager, ClientOptions};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Builder for the account manager.
pub struct AccountManagerBuilder {
    client_options: Option<ClientOptions>,
    coin_type: Option<u32>,
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub(crate) secret_manager: Option<Arc<RwLock<SecretManager>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "storage")]
pub struct StorageOptions {
    pub(crate) storage_path: PathBuf,
    pub(crate) storage_file_name: Option<String>,
    pub(crate) storage_encryption_key: Option<[u8; 32]>,
    pub(crate) manager_store: ManagerStorage,
}

#[cfg(feature = "storage")]
impl Default for StorageOptions {
    fn default() -> Self {
        StorageOptions {
            storage_path: default_storage_path().into(),
            storage_file_name: None,
            storage_encryption_key: None,
            manager_store: ManagerStorage::default(),
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Self {
            secret_manager: None,
            ..Default::default()
        }
    }

    /// Set the client options for the core nodes.
    pub fn with_client_options(mut self, client_options: ClientOptions) -> Self {
        self.client_options.replace(client_options);
        self
    }

    /// Set the coin type for the account manager. Registered coin types can be found at https://github.com/satoshilabs/slips/blob/master/slip-0044.md.
    pub fn with_coin_type(mut self, coin_type: u32) -> Self {
        self.coin_type.replace(coin_type);
        self
    }

    /// Set the secret_manager to be used.
    pub fn with_secret_manager(mut self, secret_manager: SecretManager) -> Self {
        self.secret_manager.replace(Arc::new(RwLock::new(secret_manager)));
        self
    }

    /// Set the secret_manager to be used wrapped in an Arc<RwLock<>> so it can be cloned and mutated also outside of
    /// the AccountManager.
    pub fn with_secret_manager_arc(mut self, secret_manager: Arc<RwLock<SecretManager>>) -> Self {
        self.secret_manager.replace(secret_manager);
        self
    }

    #[cfg(feature = "storage")]
    /// Set the storage path to be used.
    pub fn with_storage_path(mut self, path: &str) -> Self {
        self.storage_options = Some(StorageOptions {
            storage_path: path.into(),
            ..Default::default()
        });
        self
    }

    /// Builds the account manager
    #[allow(unreachable_code, unused_mut)]
    pub async fn finish(mut self) -> crate::Result<AccountManager> {
        log::debug!("[AccountManagerBuilder]");

        #[cfg(feature = "storage")]
        let storage_options = self.storage_options.clone().unwrap_or_default();
        #[cfg(feature = "storage")]
        // Check if the db exists and if not, return an error if one parameter is missing, because otherwise the db
        // would be created with an empty parameter which just leads to errors later
        if !storage_options.storage_path.is_dir() {
            if self.client_options.is_none() {
                return Err(crate::Error::MissingParameter("client_options"));
            }
            if self.coin_type.is_none() {
                return Err(crate::Error::MissingParameter("coin_type"));
            }
            if self.secret_manager.is_none() {
                return Err(crate::Error::MissingParameter("secret_manager"));
            }
        }
        #[cfg(all(feature = "rocksdb", feature = "storage"))]
        let storage =
            crate::storage::adapter::rocksdb::RocksdbStorageAdapter::new(storage_options.storage_path.clone())?;
        #[cfg(all(not(feature = "rocksdb"), feature = "storage"))]
        let storage = Memory::default();

        #[cfg(feature = "storage")]
        let storage_manager = crate::storage::manager::new_storage_manager(
            None,
            Box::new(storage) as Box<dyn crate::storage::adapter::StorageAdapter + Send + Sync>,
        )
        .await?;
        #[cfg(feature = "storage")]
        {
            let manager_builder = storage_manager.lock().await.get_account_manager_data().await.ok();
            let (client_options, secret_manager, coin_type) = match manager_builder {
                Some(data) => {
                    // prioritise provided client_options and secret_manager over stored ones
                    let client_options = match self.client_options {
                        Some(options) => options,
                        None => data
                            .client_options
                            .ok_or(crate::Error::MissingParameter("client_options"))?,
                    };
                    let secret_manager = match self.secret_manager {
                        Some(secret_manager) => secret_manager,
                        None => data
                            .secret_manager
                            .ok_or(crate::Error::MissingParameter("secret_manager"))?,
                    };
                    let coin_type = match self.coin_type {
                        Some(coin_type) => coin_type,
                        None => data
                            .coin_type
                            .ok_or(crate::Error::MissingParameter("coin_type (IOTA: 4218, Shimmer: 4219)"))?,
                    };
                    (client_options, secret_manager, coin_type)
                }
                // If no account manager data exist, we will set it
                None => {
                    // Store account manager data in storage
                    storage_manager.lock().await.save_account_manager_data(&self).await?;
                    (
                        self.client_options
                            .ok_or(crate::Error::MissingParameter("client_options"))?,
                        self.secret_manager
                            .ok_or(crate::Error::MissingParameter("secret_manager"))?,
                        self.coin_type
                            .ok_or(crate::Error::MissingParameter("coin_type (IOTA: 4218, Shimmer: 4219)"))?,
                    )
                }
            };

            let client = client_options.clone().finish()?;

            let accounts = storage_manager.lock().await.get_accounts().await.unwrap_or_default();

            #[cfg(feature = "events")]
            let event_emitter = Arc::new(Mutex::new(EventEmitter::new()));

            return Ok(AccountManager {
                #[cfg(not(feature = "events"))]
                accounts: Arc::new(RwLock::new(
                    accounts
                        .into_iter()
                        .map(|a| AccountHandle::new(a, client.clone(), secret_manager.clone(), storage_manager.clone()))
                        .collect(),
                )),
                #[cfg(feature = "events")]
                accounts: Arc::new(RwLock::new(
                    accounts
                        .into_iter()
                        .map(|a| {
                            AccountHandle::new(
                                a,
                                client.clone(),
                                secret_manager.clone(),
                                event_emitter.clone(),
                                storage_manager.clone(),
                            )
                        })
                        .collect(),
                )),
                background_syncing_status: Arc::new(AtomicUsize::new(0)),
                client_options: Arc::new(RwLock::new(client_options)),
                coin_type: Arc::new(AtomicU32::new(coin_type)),
                secret_manager,
                #[cfg(feature = "events")]
                event_emitter,
                storage_options,
                storage_manager,
            });
        }

        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_status: Arc::new(AtomicUsize::new(0)),
            client_options: Arc::new(RwLock::new(
                self.client_options
                    .ok_or(crate::Error::MissingParameter("client_options"))?,
            )),
            coin_type: Arc::new(AtomicU32::new(
                self.coin_type
                    .ok_or(crate::Error::MissingParameter("coin_type (IOTA: 4218, Shimmer: 4219)"))?,
            )),
            secret_manager: self
                .secret_manager
                .ok_or(crate::Error::MissingParameter("secret_manager"))?,
            #[cfg(feature = "events")]
            event_emitter: Arc::new(Mutex::new(EventEmitter::new())),
            #[cfg(feature = "storage")]
            storage_options: StorageOptions { ..Default::default() },
            #[cfg(feature = "storage")]
            storage_manager,
        })
    }

    #[cfg(feature = "storage")]
    pub(crate) async fn from_account_manager(account_manager: &AccountManager) -> Self {
        Self {
            client_options: Some(account_manager.client_options.read().await.clone()),
            coin_type: Some(account_manager.coin_type.load(Ordering::Relaxed)),
            storage_options: Some(account_manager.storage_options.clone()),
            secret_manager: Some(account_manager.secret_manager.clone()),
        }
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "storage")]
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};

use iota_client::secret::SecretManagerType;
use serde::{Deserialize, Serialize};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use crate::account::handle::AccountHandle;
#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::constants::ROCKSDB_FOLDERNAME;
#[cfg(feature = "storage")]
use crate::storage::manager::ManagerStorage;
use crate::{account_manager::AccountManager, ClientOptions};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Builder for the account manager.
pub struct AccountManagerBuilder {
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    client_options: Option<ClientOptions>,
    #[serde(default, skip_serializing, skip_deserializing)]
    secret_manager: Option<Arc<RwLock<SecretManagerType>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "storage")]
pub struct StorageOptions {
    pub(crate) storage_path: PathBuf,
    pub(crate) storage_file_name: Option<String>,
    // storage: ManagerStorage,
    pub(crate) storage_encryption_key: Option<[u8; 32]>,
    pub(crate) manager_store: ManagerStorage,
}

#[cfg(feature = "storage")]
impl Default for StorageOptions {
    fn default() -> Self {
        StorageOptions {
            storage_path: ROCKSDB_FOLDERNAME.into(),
            storage_file_name: None,
            storage_encryption_key: None,
            manager_store: ManagerStorage::Rocksdb,
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

    /// Set the IOTA client options.
    pub fn with_client_options(mut self, client_options: ClientOptions) -> Self {
        self.client_options.replace(client_options);
        self
    }

    /// Set the secret_manager to be used.
    pub fn with_secret_manager(mut self, secret_manager: SecretManagerType) -> Self {
        self.secret_manager.replace(Arc::new(RwLock::new(secret_manager)));
        self
    }

    /// Set the secret_manager to be used wrapped in an Arc<RwLock<>> so it can be cloned and mutated also outside of
    /// the AccountManager.
    pub fn with_secret_manager_arc(mut self, secret_manager: Arc<RwLock<SecretManagerType>>) -> Self {
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
    #[allow(unreachable_code)]
    pub async fn finish(self) -> crate::Result<AccountManager> {
        #[cfg(feature = "storage")]
        let storage_options = self.storage_options.clone().unwrap_or_default();
        #[cfg(feature = "storage")]
        let storage =
            crate::storage::adapter::rocksdb::RocksdbStorageAdapter::new(storage_options.storage_path.clone())?;
        #[cfg(feature = "storage")]
        let storage_manager = crate::storage::manager::new_storage_manager(
            None,
            Box::new(storage) as Box<dyn crate::storage::adapter::StorageAdapter + Send + Sync>,
        )
        .await?;
        #[cfg(feature = "storage")]
        {
            let manager_builder = storage_manager.lock().await.get_account_manager_data().await.ok();

            let (client_options, secret_manager) = match manager_builder {
                Some(data) => {
                    let client_options = match data.client_options {
                        Some(options) => options,
                        None => self
                            .client_options
                            .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
                    };
                    (
                        client_options,
                        // todo: can we get this from the read data? Maybe just with type and path for Stronghold?
                        self.secret_manager
                            .ok_or(crate::Error::MissingParameter("secret_manager"))?,
                    )
                }
                // If no account manager data exist, we will set it
                None => {
                    // Store account manager data in storage
                    storage_manager.lock().await.save_account_manager_data(&self).await?;
                    (
                        self.client_options
                            .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
                        self.secret_manager
                            .ok_or(crate::Error::MissingParameter("secret_manager"))?,
                    )
                }
            };
            let client = client_options.clone().finish().await?;

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
                    .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
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
}

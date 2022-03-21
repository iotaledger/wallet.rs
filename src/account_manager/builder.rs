// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::ManagerStorage;
use crate::{account::handle::AccountHandle, account_manager::AccountManager, ClientOptions};

use iota_client::signing::SignerHandle;
use serde::{Deserialize, Serialize};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Builder for the account manager.
pub struct AccountManagerBuilder {
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    client_options: Option<ClientOptions>,
    #[serde(default, skip_serializing, skip_deserializing)]
    signer: Option<SignerHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "storage")]
pub struct StorageOptions {
    pub(crate) storage_folder: PathBuf,
    pub(crate) storage_file_name: Option<String>,
    // storage: ManagerStorage,
    pub(crate) storage_encryption_key: Option<[u8; 32]>,
    pub(crate) manager_store: ManagerStorage,
}
impl Default for StorageOptions {
    fn default() -> Self {
        StorageOptions {
            storage_folder: "walletdb".into(),
            storage_file_name: None,
            storage_encryption_key: None,
            manager_store: ManagerStorage::Rocksdb,
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new(signer: SignerHandle) -> Self {
        Self {
            signer: Some(signer),
            ..Default::default()
        }
    }

    /// Set the IOTA client options.
    pub fn with_client_options(mut self, client_options: ClientOptions) -> Self {
        self.client_options.replace(client_options);
        self
    }

    /// Set the signer to be used.
    pub fn with_signer(mut self, signer: SignerHandle) -> Self {
        self.signer.replace(signer);
        self
    }

    #[cfg(feature = "storage")]
    /// Set the storage folder to be used.
    pub fn with_storage_folder(mut self, folder: &str) -> Self {
        self.storage_options = Some(StorageOptions {
            storage_folder: folder.into(),
            ..Default::default()
        });
        self
    }

    /// Builds the account manager
    #[allow(unreachable_code)]
    pub async fn finish(self) -> crate::Result<AccountManager> {
        #[cfg(feature = "storage")]
        {
            let storage_options = self.storage_options.clone().unwrap_or_default();

            let storage =
                crate::storage::adapter::rocksdb::RocksdbStorageAdapter::new(storage_options.storage_folder.clone())?;
            crate::storage::manager::set(
                storage_options.storage_folder.as_path(),
                None,
                Box::new(storage) as Box<dyn crate::storage::adapter::StorageAdapter + Send + Sync>,
            )
            .await?;

            let data = crate::storage::manager::load_account_manager(
                storage_options.manager_store.clone(),
                storage_options.storage_folder.clone(),
                storage_options.storage_file_name.clone(),
            )
            .await?;

            let (client_options, signer) = match data.0 {
                Some(data) => (
                    data.client_options
                        .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
                    // todo: can we get this from the read data? Maybe just with type and path for Stronghold?
                    self.signer.ok_or(crate::Error::MissingParameter("Signer"))?,
                ),
                // If no account manager data exist, we will set it
                None => {
                    // Store account manager data in storage
                    crate::storage::manager::get()
                        .await?
                        .lock()
                        .await
                        .save_account_manager_data(&self)
                        .await?;
                    (
                        self.client_options
                            .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
                        self.signer.ok_or(crate::Error::MissingParameter("Signer"))?,
                    )
                }
            };
            let client = client_options.clone().finish().await?;

            #[cfg(feature = "events")]
            let event_emitter = Arc::new(Mutex::new(EventEmitter::new()));

            return Ok(AccountManager {
                #[cfg(not(feature = "events"))]
                accounts: Arc::new(RwLock::new(data.1.into_iter().map(AccountHandle::new).collect())),
                #[cfg(feature = "events")]
                accounts: Arc::new(RwLock::new(
                    data.1
                        .into_iter()
                        .map(|a| AccountHandle::new(a, client.clone(), signer.clone(), event_emitter.clone()))
                        .collect(),
                )),
                background_syncing_status: Arc::new(AtomicUsize::new(0)),
                client_options: Arc::new(RwLock::new(client_options)),
                signer,
                #[cfg(feature = "events")]
                event_emitter,
                storage_options,
            });
        }
        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_status: Arc::new(AtomicUsize::new(0)),
            client_options: Arc::new(RwLock::new(
                self.client_options
                    .ok_or(crate::Error::MissingParameter("ClientOptions"))?,
            )),
            signer: self.signer.ok_or(crate::Error::MissingParameter("Signer"))?,
            #[cfg(feature = "events")]
            event_emitter: Arc::new(Mutex::new(EventEmitter::new())),
            #[cfg(feature = "storage")]
            storage_options: StorageOptions { ..Default::default() },
        })
    }
}

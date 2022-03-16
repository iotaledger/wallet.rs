// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::ManagerStorage;
use crate::{
    account::handle::AccountHandle,
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    ClientOptions,
};

use iota_client::signing::SignerHandle;
use serde::{Deserialize, Serialize};
#[cfg(feature = "events")]
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[cfg(feature = "storage")]
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Builder for the account manager.
pub struct AccountManagerBuilder {
    #[cfg(feature = "storage")]
    storage_options: Option<StorageOptions>,
    client_options: ClientOptions,
    #[serde(default, skip_serializing, skip_deserializing)]
    signer: Option<SignerHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "storage")]
pub struct StorageOptions {
    storage_folder: PathBuf,
    storage_file_name: Option<String>,
    // storage: ManagerStorage,
    storage_encryption_key: Option<[u8; 32]>,
    manager_store: ManagerStorage,
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

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        Self {
            #[cfg(feature = "storage")]
            storage_options: None,
            client_options: ClientOptions::new()
                // .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
                // .unwrap()
                // .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")
                // .unwrap()
                // .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")
                // .unwrap()
                // .with_node("https://chrysalis-nodes.iota.org/")?
                .with_node("http://localhost:14265")
                .unwrap()
                .with_node_sync_disabled(),
            signer: None,
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
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }
    /// Set the signer to be used.
    pub fn with_signer(mut self, signer: SignerHandle) -> Self {
        self.signer.replace(signer);
        self
    }
    /// Set the signer type to be used.
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
        // todo remove later, only called now during development
        // init_logger("wallet.log", LevelFilter::Debug)?;
        #[cfg(feature = "storage")]
        {
            let storage_folder = self.storage_options.unwrap_or_default().storage_folder;
            let options = StorageOptions {
                storage_folder: storage_folder.clone(),
                storage_file_name: None,
                // storage: ManagerStorage,
                storage_encryption_key: None,
                manager_store: ManagerStorage::Rocksdb,
            };
            let storage = crate::storage::adapter::rocksdb::RocksdbStorageAdapter::new(storage_folder.clone())?;
            crate::storage::manager::set(
                storage_folder.as_path(),
                None,
                Box::new(storage) as Box<dyn crate::storage::adapter::StorageAdapter + Send + Sync>,
            )
            .await?;
            let data = crate::storage::manager::load_account_manager(
                options.manager_store,
                options.storage_folder,
                options.storage_file_name,
            )
            .await?;
            let (client_options, signer) = match data.0 {
                Some(data) => (data.client_options, data.signer.expect("Missing signer")),
                None => (self.client_options, self.signer.expect("Missing signer")),
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
            });
        }
        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_status: Arc::new(AtomicUsize::new(0)),
            client_options: Arc::new(RwLock::new(self.client_options)),
            signer: self.signer.expect("Missing signer"),
            #[cfg(feature = "events")]
            event_emitter: Arc::new(Mutex::new(EventEmitter::new())),
        })
    }
}

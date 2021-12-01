// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::EventEmitter;
#[cfg(feature = "storage")]
use crate::storage::manager::ManagerStorage;
use crate::{
    account::handle::AccountHandle,
    account_manager::AccountManager,
    client::options::{ClientOptions, ClientOptionsBuilder},
    signing::SignerType,
};

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
    signer_type: SignerType,
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
            client_options: ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
                .unwrap()
                .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")
                .unwrap()
                .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")
                .unwrap()
                // .with_node("https://chrysalis-nodes.iota.org/")?
                // .with_node("http://localhost:14265")?
                .with_node_sync_disabled()
                .finish()
                .unwrap(),
            #[cfg(feature = "stronghold")]
            signer_type: SignerType::Stronghold,
            #[cfg(all(feature = "mnemonic", not(feature = "stronghold")))]
            signer_type: SignerType::Mnemonic,
            #[cfg(not(any(feature = "mnemonic", feature = "stronghold")))]
            signer_type: SignerType::Custom("Signer unintialized".to_string()),
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }
    /// Set the IOTA client options.
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }
    /// Set the signer type to be used.
    pub fn with_signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type = signer_type;
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
        crate::signing::set_signer(self.signer_type.clone());
        crate::client::set_client(self.client_options.clone()).await?;

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
            let (client_options, signer_type) = match data.0 {
                Some(data) => (data.client_options, data.signer_type),
                None => (self.client_options, self.signer_type),
            };
            #[cfg(feature = "events")]
            let event_emitter = Arc::new(Mutex::new(EventEmitter::new()));
            return Ok(AccountManager {
                #[cfg(not(feature = "events"))]
                accounts: Arc::new(RwLock::new(data.1.into_iter().map(AccountHandle::new).collect())),
                #[cfg(feature = "events")]
                accounts: Arc::new(RwLock::new(
                    data.1
                        .into_iter()
                        .map(|a| AccountHandle::new(a, event_emitter.clone()))
                        .collect(),
                )),
                background_syncing_status: Arc::new(AtomicUsize::new(0)),
                client_options: Arc::new(RwLock::new(client_options)),
                signer_type,
                #[cfg(feature = "events")]
                event_emitter,
            });
        }
        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(Vec::new())),
            background_syncing_status: Arc::new(AtomicUsize::new(0)),
            client_options: Arc::new(RwLock::new(self.client_options)),
            signer_type: self.signer_type,
            #[cfg(feature = "events")]
            event_emitter: Arc::new(Mutex::new(EventEmitter::new())),
        })
    }
}

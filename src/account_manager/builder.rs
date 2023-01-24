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
#[cfg(feature = "storage")]
use crate::storage::constants::ROCKSDB_FOLDERNAME;
#[cfg(feature = "storage")]
use crate::storage::manager::ManagerStorage;
use crate::{account::handle::AccountHandle, account_manager::AccountManager, ClientOptions};

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
            let read_manager_builder = storage_manager.lock().await.get_account_manager_data().await.ok();
        #[cfg(not(feature = "storage"))]
            let read_manager_builder: Option<AccountManagerBuilder> = None;

        // prioritise provided client_options and secret_manager over stored ones
        let new_provided_client_options = if self.client_options.is_none() {
            let loaded_client_options = read_manager_builder
                .as_ref()
                .and_then(|data| data.client_options.clone())
                .ok_or(crate::Error::MissingParameter("client_options"))?;

            // Update self so it gets used and stored again
            self.client_options.replace(loaded_client_options);
            false
        } else {
            true
        };

        if self.secret_manager.is_none() {
            let secret_manager = read_manager_builder
                .as_ref()
                .and_then(|data| data.secret_manager.clone())
                .ok_or(crate::Error::MissingParameter("secret_manager"))?;

            // Update self so it gets used and stored again
            self.secret_manager.replace(secret_manager);
        }

        if self.coin_type.is_none() {
            let coin_type = read_manager_builder
                .and_then(|data| data.coin_type)
                .ok_or(crate::Error::MissingParameter("coin_type (IOTA: 4218, Shimmer: 4219)"))?;

            // Update self so it gets used and stored again
            self.coin_type.replace(coin_type);
        }

        #[cfg(feature = "storage")]
        // Store account manager data in storage
        storage_manager.lock().await.save_account_manager_data(&self).await?;

        let client = self
            .client_options
            .clone()
            .ok_or(crate::Error::MissingParameter("client_options"))?
            .finish()?;

        #[cfg(feature = "events")]
            let event_emitter = Arc::new(Mutex::new(EventEmitter::new()));

        #[cfg(feature = "storage")]
            let accounts = storage_manager.lock().await.get_accounts().await.unwrap_or_default();
        #[cfg(not(feature = "storage"))]
            let accounts = Vec::new();
        let mut account_handles: Vec<AccountHandle> = accounts
            .into_iter()
            .map(|a| {
                AccountHandle::new(
                    a,
                    client.clone(),
                    self.secret_manager
                        .clone()
                        .expect("secret_manager needs to be provided"),
                    #[cfg(feature = "events")]
                        event_emitter.clone(),
                    #[cfg(feature = "storage")]
                        storage_manager.clone(),
                )
            })
            .collect::<_>();

        // If they are new, we need to update the accounts because the bech32 HRP might have changed.
        // In the other case it was loaded from the database and addresses are up to date.
        if new_provided_client_options {
            for account in account_handles.iter_mut() {
                account.update_account_with_new_client(client.clone()).await?;
            }
        }

        Ok(AccountManager {
            accounts: Arc::new(RwLock::new(account_handles)),
            background_syncing_status: Arc::new(AtomicUsize::new(0)),
            client_options: Arc::new(RwLock::new(
                self.client_options
                    .clone()
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
            event_emitter,
            #[cfg(feature = "storage")]
            storage_options,
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
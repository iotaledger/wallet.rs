// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{str::FromStr, sync::Arc};

use crypto::ciphers::chacha;
use iota_client::secret::{SecretManager, SecretManagerDto};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::{
    account::Account,
    account_manager::builder::AccountManagerBuilder,
    storage::{constants::*, Storage, StorageAdapter},
};

/// The storage used by the manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ManagerStorage {
    /// RocksDB storage.
    #[cfg(feature = "rocksdb")]
    Rocksdb,
    /// Storage backed by a Map in memory.
    Memory,
    /// Wasm storage.
    #[cfg(target_family = "wasm")]
    Wasm,
}

impl Default for ManagerStorage {
    fn default() -> Self {
        #[cfg(feature = "rocksdb")]
        return Self::Rocksdb;
        #[cfg(target_family = "wasm")]
        return Self::Wasm;
        #[cfg(not(any(feature = "rocksdb", target_family = "wasm")))]
        Self::Memory
    }
}

pub(crate) type StorageManagerHandle = Arc<Mutex<StorageManager>>;

/// Sets the storage adapter.
pub(crate) async fn new_storage_manager(
    encryption_key: Option<[u8; 32]>,
    storage: Box<dyn StorageAdapter + Send + Sync + 'static>,
) -> crate::Result<StorageManagerHandle> {
    let mut storage = Storage {
        inner: storage,
        encryption_key,
    };
    // Get the db version or set it
    let db_schema_version = storage.get(DATABASE_SCHEMA_VERSION_KEY).await?;
    if let Some(db_schema_version) = db_schema_version {
        let db_schema_version = u8::from_str(&db_schema_version)
            .map_err(|_| crate::Error::Storage("invalid db_schema_version".to_string()))?;
        if db_schema_version != DATABASE_SCHEMA_VERSION {
            return Err(crate::Error::Storage(format!(
                "unsupported database schema version {db_schema_version}"
            )));
        }
    } else {
        storage
            .set(DATABASE_SCHEMA_VERSION_KEY, DATABASE_SCHEMA_VERSION)
            .await?;
    };

    let account_indexes = match storage.get(ACCOUNTS_INDEXATION_KEY).await? {
        Some(account_indexes) => serde_json::from_str(&account_indexes)?,
        None => Vec::new(),
    };
    let storage_manager = StorageManager {
        storage,
        account_indexes,
    };

    Ok(Arc::new(Mutex::new(storage_manager)))
}

/// Storage manager
#[derive(Debug)]
pub struct StorageManager {
    pub(crate) storage: Storage,
    // account indexes for accounts in the database
    account_indexes: Vec<u32>,
}

impl StorageManager {
    pub fn id(&self) -> &'static str {
        self.storage.id()
    }

    #[cfg(test)]
    pub fn is_encrypted(&self) -> bool {
        self.storage.encryption_key.is_some()
    }

    pub async fn get(&self, key: &str) -> crate::Result<Option<String>> {
        self.storage.get(key).await
    }

    pub async fn save_account_manager_data(
        &mut self,
        account_manager_builder: &AccountManagerBuilder,
    ) -> crate::Result<()> {
        log::debug!("save_account_manager_data");
        self.storage
            .set(ACCOUNT_MANAGER_INDEXATION_KEY, account_manager_builder)
            .await?;

        if let Some(secret_manager) = &account_manager_builder.secret_manager {
            let secret_manager = secret_manager.read().await;
            let secret_manager_dto = SecretManagerDto::from(&*secret_manager);
            // Only store secret_managers that aren't SecretManagerDto::Mnemonic, because there the Seed can't be
            // serialized, so we can't create the SecretManager again
            match secret_manager_dto {
                SecretManagerDto::Mnemonic(_) => {}
                _ => {
                    self.storage.set(SECRET_MANAGER_KEY, secret_manager_dto).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_account_manager_data(&self) -> crate::Result<Option<AccountManagerBuilder>> {
        log::debug!("get_account_manager_data");
        if let Some(data) = self.storage.get(ACCOUNT_MANAGER_INDEXATION_KEY).await? {
            log::debug!("get_account_manager_data {data:?}");
            let mut builder: AccountManagerBuilder = serde_json::from_str(&data)?;

            if let Some(data) = self.storage.get(SECRET_MANAGER_KEY).await? {
                log::debug!("get_secret_manager {data}");
                let secret_manager_dto: SecretManagerDto = serde_json::from_str(&data)?;
                // Only secret_managers that aren't SecretManagerDto::Mnemonic can be restored, because there the Seed
                // can't be serialized, so we can't create the SecretManager again
                match secret_manager_dto {
                    SecretManagerDto::Mnemonic(_) => {}
                    _ => {
                        let secret_manager = SecretManager::try_from(&secret_manager_dto)?;
                        builder.secret_manager = Some(Arc::new(RwLock::new(secret_manager)));
                    }
                }
            }
            Ok(Some(builder))
        } else {
            Ok(None)
        }
    }

    pub async fn get_accounts(&mut self) -> crate::Result<Vec<Account>> {
        if let Some(record) = self.storage.get(ACCOUNTS_INDEXATION_KEY).await? {
            if self.account_indexes.is_empty() {
                self.account_indexes = serde_json::from_str(&record)?;
            }
        } else {
            return Ok(Vec::new());
        }

        let mut accounts = Vec::new();
        for account_index in self.account_indexes.clone() {
            // PANIC: we assume that ACCOUNTS_INDEXATION_KEY and the different indexes are set together and
            // ACCOUNTS_INDEXATION_KEY has already been checked.
            accounts.push(
                self.get(&format!("{ACCOUNT_INDEXATION_KEY}{account_index}"))
                    .await?
                    .unwrap(),
            );
        }

        parse_accounts(&accounts, &self.storage.encryption_key)
    }

    pub async fn save_account(&mut self, account: &Account) -> crate::Result<()> {
        // Only add account index if not already present
        if !self.account_indexes.contains(account.index()) {
            self.account_indexes.push(*account.index());
        }

        self.storage
            .set(ACCOUNTS_INDEXATION_KEY, self.account_indexes.clone())
            .await?;
        self.storage
            .set(&format!("{ACCOUNT_INDEXATION_KEY}{}", account.index()), account)
            .await
    }

    pub async fn remove_account(&mut self, account_index: u32) -> crate::Result<()> {
        self.storage
            .remove(&format!("{ACCOUNT_INDEXATION_KEY}{account_index}"))
            .await?;
        self.account_indexes.retain(|a| a != &account_index);
        self.storage
            .set(ACCOUNTS_INDEXATION_KEY, self.account_indexes.clone())
            .await
    }
}

// Parse accounts from strings and decrypt them first if necessary
fn parse_accounts(accounts: &[String], encryption_key: &Option<[u8; 32]>) -> crate::Result<Vec<Account>> {
    let mut parsed_accounts: Vec<Account> = Vec::new();
    for account in accounts {
        let account_json = if account.starts_with('{') {
            Some(account.to_string())
        } else if let Some(key) = encryption_key {
            Some(String::from_utf8_lossy(&chacha::aead_decrypt(key, account.as_bytes())?).into_owned())
        } else {
            None
        };
        if let Some(json) = account_json {
            let acc = serde_json::from_str::<Account>(&json)?;
            parsed_accounts.push(acc);
        } else {
            return Err(crate::Error::StorageIsEncrypted);
        }
    }
    Ok(parsed_accounts)
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    account_manager::builder::AccountManagerBuilder,
    storage::{constants::*, decrypt_record, Storage, StorageAdapter},
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use std::{path::Path, sync::Arc};

/// The storage used by the manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ManagerStorage {
    #[cfg(feature = "stronghold")]
    /// Stronghold storage.
    Stronghold,
    /// RocksDB storage.
    Rocksdb,
}

pub(crate) type StorageManagerHandle = Arc<Mutex<StorageManager>>;

// // todo get other manager data like client options and signertype
// pub(crate) async fn load_account_manager(
//     manager_storage: ManagerStorage,
//     storage_path: PathBuf,
// ) -> crate::Result<(Option<AccountManagerBuilder>, Vec<Account>)> {
//     let (storage, storage_file_path, is_stronghold): (Option<Box<dyn StorageAdapter + Send + Sync>>, PathBuf, bool) =
//         match manager_storage {
//             #[cfg(feature = "stronghold")]
//             ManagerStorage::Stronghold => {
//                 todo!()
//                 // let path = storage_folder.join(storage_file_name.as_deref().unwrap_or(STRONGHOLD_FILENAME));
//                 // fs::create_dir_all(&storage_folder)?;
//                 // let storage = crate::storage::adapter::stronghold::StrongholdStorageAdapter::new(&path)?;
//                 // (
//                 //     Some(Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>),
//                 //     path,
//                 //     true,
//                 // )
//             }
//             ManagerStorage::Rocksdb => {
//                 // let path = storage_folder.join(storage_file_name.as_deref().unwrap_or(ROCKSDB_FOLDERNAME));
//                 fs::create_dir_all(&storage_path)?;
//                 // rocksdb storage already exists; no need to create a new instance
//                 let storage = if crate::storage::manager::get().await.is_ok() {
//                     None
//                 } else {
//                     let storage = crate::storage::adapter::rocksdb::RocksdbStorageAdapter::new(&storage_path)?;
//                     Some(Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>)
//                 };
//                 (storage, storage_path, false)
//             }
//         };

//     let manager = crate::storage::manager::get().await?;
//     let mut storage_manager = manager.lock().await;
//     let manager_builder = storage_manager.get_account_manager_data().await.ok();
//     let accounts = storage_manager.get_accounts().await.unwrap_or_default();

//     Ok((manager_builder, accounts))
// }

/// Sets the storage adapter.
pub(crate) async fn new_storage_manager<P: AsRef<Path>>(
    storage_path: P,
    encryption_key: Option<[u8; 32]>,
    storage: Box<dyn StorageAdapter + Send + Sync + 'static>,
) -> crate::Result<StorageManagerHandle> {
    let storage = Storage {
        storage_path: storage_path.as_ref().to_path_buf(),
        inner: storage,
        encryption_key,
    };
    let account_indexes = match storage.get(ACCOUNTS_INDEXATION_KEY).await {
        Ok(account_indexes) => serde_json::from_str(&account_indexes)?,
        Err(_) => Vec::new(),
    };
    let storage_manager = StorageManager {
        storage,
        account_indexes,
    };

    Ok(Arc::new(Mutex::new(storage_manager)))
}

#[derive(Debug)]
/// Storage manager
pub struct StorageManager {
    storage: Storage,
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

    pub async fn get(&self, key: &str) -> crate::Result<String> {
        self.storage.get(key).await
    }

    pub async fn save_account_manager_data(
        &mut self,
        account_manager_builder: &AccountManagerBuilder,
    ) -> crate::Result<()> {
        log::debug!("save_account_manager_data");
        self.storage
            .set(ACCOUNT_MANAGER_INDEXATION_KEY, account_manager_builder)
            .await
    }

    pub async fn get_account_manager_data(&self) -> crate::Result<AccountManagerBuilder> {
        log::debug!("get_account_manager_data");
        let data = self.storage.get(ACCOUNT_MANAGER_INDEXATION_KEY).await?;
        let builder: AccountManagerBuilder = serde_json::from_str(&data)?;
        Ok(builder)
    }

    pub async fn get_accounts(&mut self) -> crate::Result<Vec<Account>> {
        if self.account_indexes.is_empty() {
            if let Ok(record) = self.storage.get(ACCOUNTS_INDEXATION_KEY).await {
                self.account_indexes = serde_json::from_str(&record)?;
            }
        }

        let mut accounts = Vec::new();
        for account_index in self.account_indexes.clone() {
            accounts.push(
                self.get(&format!("{}{}", ACCOUNT_INDEXATION_KEY, account_index))
                    .await?,
            );
        }
        parse_accounts(&accounts, &self.storage.encryption_key)
    }

    pub async fn save_account(&mut self, account: &Account) -> crate::Result<()> {
        self.account_indexes.push(*account.index());
        self.storage
            .set(ACCOUNTS_INDEXATION_KEY, self.account_indexes.clone())
            .await?;
        self.storage
            .set(&format!("{}{}", ACCOUNT_INDEXATION_KEY, account.index()), account)
            .await
    }

    pub async fn remove_account(&mut self, account_index: u32) -> crate::Result<()> {
        self.storage
            .remove(&format!("{}{}", ACCOUNT_INDEXATION_KEY, account_index))
            .await?;
        self.account_indexes.retain(|a| a == &account_index);
        self.storage
            .set(ACCOUNTS_INDEXATION_KEY, self.account_indexes.clone())
            .await
    }

    #[cfg(feature = "ledger-nano")]
    // used for ledger accounts to verify that the same menmonic is used for all accounts
    pub async fn save_first_ledger_address(
        &mut self,
        address: &iota_client::bee_message::address::Address,
    ) -> crate::Result<()> {
        self.storage.set(FIRST_LEDGER_ADDRESS_KEY, address).await?;
        Ok(())
    }

    #[cfg(feature = "ledger-nano")]
    pub async fn get_first_ledger_address(&self) -> crate::Result<iota_client::bee_message::address::Address> {
        let address: iota_client::bee_message::address::Address =
            serde_json::from_str(&self.storage.get(FIRST_LEDGER_ADDRESS_KEY).await?)?;
        Ok(address)
    }
}

// Parse accounts from strings and decrypt them first if necessary
fn parse_accounts(accounts: &[String], encryption_key: &Option<[u8; 32]>) -> crate::Result<Vec<Account>> {
    let mut parsed_accounts: Vec<Account> = Vec::new();
    for account in accounts {
        let account_json = if account.starts_with('{') {
            Some(account.to_string())
        } else if let Some(key) = encryption_key {
            Some(decrypt_record(account, key)?)
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

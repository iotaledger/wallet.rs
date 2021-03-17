// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use std::{num::NonZeroU64, path::PathBuf};

use iota_wallet::{
    account_manager::{
        AccountManager as AccountManagerRust, ManagerStorage as ManagerStorageRust, DEFAULT_STORAGE_FOLDER,
    },
    message::MessageId,
    signing::SignerType,
};

use crate::{
    acc::{Account, AccountInitialiser},
    client_options::ClientOptions,
    message::Message,
    Result,
};

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_FOLDER.into()
}

#[derive(Debug)]
pub enum AccountSignerType {
    Stronghold = 1,
    LedgerNano = 2,
    LedgerNanoSimulator = 3,
}

pub fn signer_type_enum_to_type(signer_type: AccountSignerType) -> SignerType {
    match signer_type {
        #[cfg(feature = "stronghold")]
        AccountSignerType::Stronghold => SignerType::Stronghold,

        #[cfg(feature = "ledger-nano")]
        AccountSignerType::LedgerNano => SignerType::LedgerNano,

        #[cfg(feature = "ledger-nano-simulator")]
        AccountSignerType::LedgerNanoSimulator => SignerType::LedgerNanoSimulator,

        // Default to Stringhold
        // TODO: Will break
        _ => SignerType::Stronghold,
    }
}

#[derive(Debug)]
pub enum ManagerStorage {
    Stronghold = 1,
    Sqlite = 2,
}

fn storage_enum_to_storage(storage: ManagerStorage) -> ManagerStorageRust {
    match storage {
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        ManagerStorage::Stronghold => ManagerStorageRust::Stronghold,

        #[cfg(feature = "sqlite-storage")]
        ManagerStorage::Sqlite => ManagerStorageRust::Sqlite,
    }
}

pub struct ManagerOptions {
    storage_path: PathBuf,
    storage_type: Option<ManagerStorageRust>,
    storage_password: Option<String>,
}

impl Default for ManagerOptions {
    fn default() -> Self {
        #[allow(unused_variables)]
        let default_storage: Option<ManagerStorageRust> = None;
        #[cfg(all(feature = "stronghold-storage", not(feature = "sqlite-storage")))]
        let default_storage = Some(ManagerStorageRust::Stronghold);
        #[cfg(all(feature = "sqlite-storage", not(feature = "stronghold-storage")))]
        let default_storage = Some(ManagerStorageRust::Sqlite);

        Self {
            storage_path: default_storage_path(),
            storage_type: default_storage,
            // polling_interval: Duration::from_millis(30_000),
            // skip_polling: false,
            storage_password: None,
        }
    }
}

impl ManagerOptions {
    pub fn set_storage_path(&mut self, storage_path: PathBuf) {
        self.storage_path = storage_path;
    }

    pub fn set_storage_type(&mut self, storage_type: ManagerStorage) {
        self.storage_type = Option::Some(storage_enum_to_storage(storage_type));
    }

    pub fn set_storage_password(&mut self, storage_password: String) {
        self.storage_password = Option::Some(storage_password);
    }
}

pub struct AccountManager {
    manager: AccountManagerRust,
}

impl AccountManager {
    pub fn new(options: ManagerOptions) -> AccountManager {
        let manager = crate::block_on(
            AccountManagerRust::builder()
                .with_storage(
                    PathBuf::from(options.storage_path),
                    options.storage_type.unwrap_or(ManagerStorageRust::Stronghold),
                    options.storage_password.as_deref(),
                )
                .expect("failed to init storage")
                .finish(),
        )
        .expect("error initializing account manager");

        AccountManager { manager: manager }
    }

    pub fn storage_path(&self) -> &PathBuf {
        self.manager.storage_path()
    }

    pub fn stop_background_sync(&mut self) -> Result<()> {
        self.manager.stop_background_sync();
        Ok(())
    }

    pub fn set_storage_password(&mut self, password: &str) -> Result<()> {
        match crate::block_on(async move { self.manager.set_storage_password(password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn set_stronghold_password(&mut self, password: &str) -> Result<()> {
        match crate::block_on(async move { self.manager.set_stronghold_password(password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn change_stronghold_password(&mut self, current_password: &str, new_password: &str) -> Result<()> {
        match crate::block_on(async move {
            self.manager
                .change_stronghold_password(current_password, new_password)
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn generate_mnemonic(&mut self) -> Result<String> {
        match self.manager.generate_mnemonic() {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(mnemonic) => Ok(mnemonic),
        }
    }

    pub fn store_mnemonic(&mut self, signer_type_enum: AccountSignerType, mnemonic: String) -> Result<()> {
        let signer_type = signer_type_enum_to_type(signer_type_enum);

        // TODO: Make optional from java possible
        let opt_mnemonic = match mnemonic.as_str() {
            "" => None,
            _ => Some(mnemonic),
        };

        match crate::block_on(async move { self.manager.store_mnemonic(signer_type, opt_mnemonic).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn verify_mnemonic(&mut self, mnemonic: String) -> Result<()> {
        match self.manager.verify_mnemonic(mnemonic) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn create_account(&self, client_options: ClientOptions) -> Result<AccountInitialiser> {
        match self.manager.create_account(client_options.get_internal()) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(initialiser) => Ok(AccountInitialiser::new(initialiser)),
        }
    }

    pub fn remove_account(&self, account_id: String) -> Result<()> {
        match crate::block_on(async move { self.manager.remove_account(account_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn get_account(&self, account_id: String) -> Result<Account> {
        match crate::block_on(async move { self.manager.get_account(account_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(acc) => Ok(Account::new_with_internal(acc)),
        }
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        match crate::block_on(async move { self.manager.get_accounts().await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(accs) => Ok(accs.iter().map(|acc| Account::new_with_internal(acc.clone())).collect()),
        }
    }

    // TODO: Do we still need synchronisers?
    // pub fn sync_accounts(&self) -> Result<AccountsSynchronizer> {
    // self.manager.sync_accounts()
    // }

    pub fn reattach(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.reattach(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(Message::new_with_internal(msg)),
        }
    }

    pub fn promote(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.promote(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(Message::new_with_internal(msg)),
        }
    }

    pub fn retry(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.retry(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(Message::new_with_internal(msg)),
        }
    }

    pub fn internal_transfer(&self, from_account_id: String, to_account_id: String, amount: u64) -> Result<Message> {
        match crate::block_on(async move {
            self.manager
                .internal_transfer(from_account_id, to_account_id, NonZeroU64::new(amount).unwrap())
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(Message::new_with_internal(msg)),
        }
    }

    #[cfg(not(any(feature = "stronghold-storage", feature = "sqlite-storage")))]
    pub fn backup(&self, _: PathBuf) -> Result<PathBuf> {
        Err(anyhow!("No storage found during compilation"))
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    pub fn backup(&self, destination: PathBuf) -> Result<PathBuf> {
        match crate::block_on(async move { self.manager.backup(destination).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(path) => Ok(path),
        }
    }

    #[cfg(not(any(feature = "stronghold-storage", feature = "sqlite-storage")))]
    pub fn import_accounts(&self, _: PathBuf, _: String) -> Result<()> {
        Err(anyhow!("No storage found during compilation"))
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    pub fn import_accounts(&mut self, source: PathBuf, stronghold_password: String) -> Result<()> {
        match crate::block_on(async move { self.manager.import_accounts(source, stronghold_password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }
}

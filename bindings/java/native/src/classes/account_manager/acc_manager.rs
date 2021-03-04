#![allow(non_snake_case)]

use std::{path::PathBuf};

use iota_wallet::{
    account_manager::{
        AccountManager as AccountManagerRust,
        ManagerStorage as ManagerStorageRust,
        DEFAULT_STORAGE_FOLDER
    },
    signing::SignerType,
};

use crate::Result;
use crate::{
    client_options::ClientOptions,
    acc::AccountInitialiser
};

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_FOLDER.into()
}

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

pub enum ManagerStorage {
    Stronghold = 1,
    Sqlite = 2,
}

fn storage_enum_to_storage(storage: ManagerStorage) -> ManagerStorageRust {
    match storage {
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        Stronghold => ManagerStorageRust::Stronghold,

        #[cfg(feature = "sqlite-storage")]
        Sqlite => ManagerStorageRust::Sqlite,

        // Default to Stringhold
        // TODO: Will break
        _ => ManagerStorageRust::Stronghold,
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
            //polling_interval: Duration::from_millis(30_000),
            //skip_polling: false,
            storage_password: None,
        }
    }
}

impl ManagerOptions {
    pub fn setStoragePath(&mut self, storage_path: PathBuf){
        println!("old storage: {:?}", &self.storage_path);
        self.storage_path = storage_path;
    }

    pub fn setStorageType(&mut self, storage_type: ManagerStorage){
        self.storage_type = Option::Some(storage_enum_to_storage(storage_type));
    }

    pub fn setStoragePassword(&mut self, storage_password: String){
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
            .finish()
        ).expect("error initializing account manager");

        AccountManager {
            manager: manager
        }
    }

    pub fn storage_path(& self) -> &PathBuf {
        self.manager.storage_path()
    }

    pub fn stopBackgroundSync(&mut self,) -> Result<()> {
        self.manager.stop_background_sync();
        Ok(())
    }

    pub fn setStoragePassword(&mut self, password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.set_storage_password(password).await
        }).expect("error setting storage password");
        Ok(())
    }

    pub fn setStrongholdPassword(&mut self, password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.set_stronghold_password(password).await
        }).expect("error setting stronghold password");
        Ok(())
    }

    pub fn changeStrongholdPassword(&mut self, current_password: &str, new_password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.change_stronghold_password(current_password, new_password).await
        }).expect("error changing stronghold password");
        Ok(())
    }

    pub fn generateMnemonic(&mut self) -> Result<String> {
        let mnemonic = self.manager.generate_mnemonic()
            .expect("error generating mnemonic");
        Ok(mnemonic)
    }

    pub fn storeMnemonic(&mut self, signer_type_enum: AccountSignerType, mnemonic: String) -> Result<()> {
        let signer_type = signer_type_enum_to_type(signer_type_enum);

        // TODO: Make optional from java possible
        let opt_mnemonic = match mnemonic.as_str() {
            "" => None,
            _ => Some(mnemonic),
        };
        
        crate::block_on(async move {
            self.manager.store_mnemonic(signer_type, opt_mnemonic).await
        }).expect("error storing mnemonic");
        Ok(())
    }

    pub fn verifyMnemonic(&mut self, mnemonic: String) -> Result<()> {
        self.manager.verify_mnemonic(mnemonic).expect("error verifying mnemonic");
        Ok(())
    }

    pub fn createAccount(&self, client_options: ClientOptions) -> Result<AccountInitialiser>{
        let initialiser = self.manager.create_account(client_options.get_internal())
            .expect("Failed to initialise accauntinitialiser");
        
        Ok(AccountInitialiser::new(initialiser))
    }
}


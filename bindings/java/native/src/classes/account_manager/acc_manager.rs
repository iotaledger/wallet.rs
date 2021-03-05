#![allow(non_snake_case)]

use std::{
    path::PathBuf,
    num::NonZeroU64,
};

use iota_wallet::{
    account_manager::{
        AccountManager as AccountManagerRust,
        ManagerStorage as ManagerStorageRust,
        DEFAULT_STORAGE_FOLDER
    },
    message::MessageId,
    signing::SignerType,
};

use crate::{
    Result,
    client_options::ClientOptions,
    acc::{
        AccountInitialiser, Account
    },
    message::Message,
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
    pub fn set_storage_path(&mut self, storage_path: PathBuf){
        println!("old storage: {:?}", &self.storage_path);
        self.storage_path = storage_path;
    }

    pub fn set_storage_type(&mut self, storage_type: ManagerStorage){
        self.storage_type = Option::Some(storage_enum_to_storage(storage_type));
    }

    pub fn set_storage_password(&mut self, storage_password: String){
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

    pub fn stop_background_sync(&mut self,) -> Result<()> {
        self.manager.stop_background_sync();
        Ok(())
    }

    pub fn set_storage_password(&mut self, password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.set_storage_password(password).await
        }).expect("error setting storage password");
        Ok(())
    }

    pub fn set_stronghold_password(&mut self, password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.set_stronghold_password(password).await
        }).expect("error setting stronghold password");
        Ok(())
    }

    pub fn change_stronghold_password(&mut self, current_password: &str, new_password: &str) -> Result<()> {
        crate::block_on(async move {
            self.manager.change_stronghold_password(current_password, new_password).await
        }).expect("error changing stronghold password");
        Ok(())
    }

    pub fn generate_mnemonic(&mut self) -> Result<String> {
        let mnemonic = self.manager.generate_mnemonic()
            .expect("error generating mnemonic");
        Ok(mnemonic)
    }

    pub fn store_mnemonic(&mut self, signer_type_enum: AccountSignerType, mnemonic: String) -> Result<()> {
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

    pub fn verify_mnemonic(&mut self, mnemonic: String) -> Result<()> {
        self.manager.verify_mnemonic(mnemonic).expect("error verifying mnemonic");
        Ok(())
    }

    pub fn create_account(&self, client_options: ClientOptions) -> Result<AccountInitialiser>{
        let initialiser = self.manager.create_account(client_options.get_internal())
            .expect("Failed to initialise accauntinitialiser");
        
        Ok(AccountInitialiser::new(initialiser))
    }

    pub fn remove_account(&self, account_id: String) -> Result<()> {
        crate::block_on(async move {
            self.manager.remove_account(account_id).await
        }).expect("error removing account");

        Ok(())
    }

    pub fn get_account(&self, account_id: String) -> Result<Account> {
        let acc = crate::block_on(async move {
            self.manager.get_account(account_id).await
        }).expect("error getting account");

        Ok(Account::new_with_internal(acc))
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        let accs = crate::block_on(async move {
            self.manager.get_accounts().await
        }).expect("error getting accounts");

        Ok(accs.iter().map(|acc| Account::new_with_internal(acc.clone()) ).collect())
    }

    //TODO: Do we still need synchronisers?
    /*
    pub fn sync_accounts(&self) -> Result<AccountsSynchronizer> {
        self.manager.sync_accounts()
    }
    */

    
    pub fn reattach(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        let msg = crate::block_on(async move {
            self.manager.reattach(account_id, &message_id).await
        }).expect("error reattaching message");

        Ok(Message::new_with_internal(msg))
    }

    pub fn promote(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        let msg = crate::block_on(async move {
            self.manager.promote(account_id, &message_id).await
        }).expect("error promoting message");

        Ok(Message::new_with_internal(msg))
    }

    pub fn retry(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        let msg = crate::block_on(async move {
            self.manager.retry(account_id, &message_id).await
        }).expect("error retrying account");

        Ok(Message::new_with_internal(msg))
    }

    pub fn internal_transfer(&self, from_account_id: String, to_account_id: String, amount: u64) -> Result<Message> {
        let msg = crate::block_on(async move {
            self.manager.internal_transfer(from_account_id, to_account_id, NonZeroU64::new(amount).unwrap()).await
        }).expect("error retrying account");

        Ok(Message::new_with_internal(msg))
    }
/*
    #[cfg(!any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    pub async fn backup(&self, destination: Path) -> crate::Result<PathBuf> {
        Err(anyhow!("No storage found during compilation"))
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    pub async fn backup(&self, destination: Path) -> crate::Result<PathBuf> {

    }

    #[cfg(!any(feature = "stronghold-storage", feature = "sqlite-storage"))]ç
    pub async fn import_accounts<S: AsRef<Path>>(
        &mut self,
        source: S,
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))] stronghold_password: String,
    ) -> crate::Result<()> {
        Err(anyhow!("No storage found during compilation"))
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    pub async fn import_accounts<S: AsRef<Path>>(
        &mut self,
        source: S,
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))] stronghold_password: String,
    ) -> crate::Result<()> {

    }
    */
}


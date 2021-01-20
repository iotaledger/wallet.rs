// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use crate::{
    account::{
        repost_message, Account, AccountHandle, AccountIdentifier, AccountInitialiser, RepostAction, SyncedAccount,
    },
    client::ClientOptions,
    event::{emit_balance_change, emit_confirmation_state_change, emit_transaction_event, TransactionEventType},
    message::{Message, MessageType, Transfer},
    signing::SignerType,
    storage::StorageAdapter,
};

use std::{
    collections::HashMap,
    convert::TryInto,
    fs,
    num::NonZeroU64,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use chrono::prelude::*;
use futures::FutureExt;
use getset::Getters;
use iota::{MessageId, Payload};
use serde::Deserialize;
use tokio::{
    sync::{
        broadcast::{channel as broadcast_channel, Receiver as BroadcastReceiver, Sender as BroadcastSender},
        RwLock,
    },
    time::interval,
};

/// The default storage folder.
pub const DEFAULT_STORAGE_FOLDER: &str = "./storage";

/// The default stronghold storage file name.
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
pub const STRONGHOLD_FILENAME: &str = "wallet.stronghold";

/// The default SQLite storage file name.
#[cfg(feature = "sqlite-storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite-storage")))]
pub const SQLITE_FILENAME: &str = "wallet.db";

pub(crate) type AccountStore = Arc<RwLock<HashMap<String, AccountHandle>>>;

/// The storage used by the manager.
#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ManagerStorage {
    /// Stronghold storage.
    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
    Stronghold,
    /// Sqlite storage.
    #[cfg(feature = "sqlite-storage")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sqlite-storage")))]
    Sqlite,
    /// Custom storage.
    #[serde(skip)]
    Custom(Box<dyn StorageAdapter + Send + Sync + 'static>),
}

#[cfg(any(feature = "stronghold", feature = "stronghold-storage", feature = "sqlite-storage"))]
fn storage_file_path(storage: &Option<ManagerStorage>, storage_path: &PathBuf) -> PathBuf {
    if storage_path.is_file() || storage_path.extension().is_some() || storage.is_none() {
        storage_path.clone()
    } else {
        match storage {
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            Some(ManagerStorage::Stronghold) => storage_path.join(STRONGHOLD_FILENAME),
            #[cfg(feature = "sqlite-storage")]
            Some(ManagerStorage::Sqlite) => storage_path.join(SQLITE_FILENAME),
            _ => storage_path.clone(),
        }
    }
}

fn storage_password_to_encryption_key(password: &str) -> [u8; 32] {
    let mut dk = [0; 64];
    // safe to unwrap (rounds > 0)
    crypto::kdfs::pbkdf::PBKDF2_HMAC_SHA512(password.as_bytes(), b"wallet.rs::storage", 100, &mut dk).unwrap();
    let key: [u8; 32] = dk[0..32][..].try_into().unwrap();
    key
}

/// Account manager builder.
pub struct AccountManagerBuilder {
    storage_path: PathBuf,
    storage: Option<ManagerStorage>,
    polling_interval: Duration,
    skip_polling: bool,
    storage_encryption_key: Option<[u8; 32]>,
}

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        #[allow(unused_variables)]
        let default_storage: Option<ManagerStorage> = None;
        #[cfg(all(feature = "stronghold-storage", not(feature = "sqlite-storage")))]
        let default_storage = Some(ManagerStorage::Stronghold);
        #[cfg(all(feature = "sqlite-storage", not(feature = "stronghold-storage")))]
        let default_storage = Some(ManagerStorage::Sqlite);

        Self {
            storage_path: PathBuf::from(DEFAULT_STORAGE_FOLDER),
            storage: default_storage,
            polling_interval: Duration::from_millis(30_000),
            skip_polling: false,
            storage_encryption_key: None,
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the storage to be used.
    pub fn with_storage(
        mut self,
        storage_path: impl AsRef<Path>,
        storage: ManagerStorage,
        password: Option<&str>,
    ) -> crate::Result<Self> {
        self.storage_path = storage_path.as_ref().to_path_buf();
        self.storage = Some(storage);
        self.storage_encryption_key = match password {
            Some(p) => Some(storage_password_to_encryption_key(p)),
            None => None,
        };
        Ok(self)
    }

    pub(crate) fn with_storage_encryption_key(mut self, key: Option<[u8; 32]>) -> Self {
        self.storage_encryption_key = key;
        self
    }

    /// Sets the polling interval.
    pub fn with_polling_interval(mut self, polling_interval: Duration) -> Self {
        self.polling_interval = polling_interval;
        self
    }

    pub(crate) fn skip_polling(mut self) -> Self {
        self.skip_polling = true;
        self
    }

    /// Builds the manager.
    pub async fn finish(self) -> crate::Result<AccountManager> {
        let (storage, storage_file_path): (Box<dyn StorageAdapter + Send + Sync>, PathBuf) =
            if let Some(storage) = self.storage {
                match storage {
                    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
                    ManagerStorage::Stronghold => {
                        let path = storage_file_path(&Some(ManagerStorage::Stronghold), &self.storage_path);
                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(&parent)?;
                        }
                        let storage = crate::storage::stronghold::StrongholdStorageAdapter::new(&path)?;
                        (Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>, path)
                    }
                    #[cfg(feature = "sqlite-storage")]
                    ManagerStorage::Sqlite => {
                        let path = storage_file_path(&Some(ManagerStorage::Sqlite), &self.storage_path);
                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(&parent)?;
                        }
                        let storage = crate::storage::sqlite::SqliteStorageAdapter::new(&path, "accounts")?;
                        (Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>, path)
                    }
                    ManagerStorage::Custom(storage) => (storage, self.storage_path.clone()),
                }
            } else {
                return Err(crate::Error::StorageAdapterNotDefined);
            };

        crate::storage::set(&storage_file_path, self.storage_encryption_key, storage).await;

        // with the stronghold storage feature, the accounts are loaded when the password is set
        #[cfg(feature = "stronghold-storage")]
        let (accounts, encrypted_accounts) = (Default::default(), Vec::new());
        #[cfg(not(feature = "stronghold-storage"))]
        let (accounts, encrypted_accounts) = AccountManager::load_accounts(&storage_file_path)
            .await
            .unwrap_or_else(|_| (AccountStore::default(), Vec::new()));

        let mut instance = AccountManager {
            storage_folder: if self.storage_path.is_file() || self.storage_path.extension().is_some() {
                match self.storage_path.parent() {
                    Some(p) => p.to_path_buf(),
                    None => self.storage_path,
                }
            } else {
                self.storage_path
            },
            storage_path: storage_file_path,
            accounts,
            stop_polling_sender: None,
            polling_handle: None,
            generated_mnemonic: None,
            encrypted_accounts,
        };

        if !self.skip_polling {
            instance.start_background_sync(self.polling_interval).await;
        }

        Ok(instance)
    }
}

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Getters)]
pub struct AccountManager {
    storage_folder: PathBuf,
    /// the path to the storage.
    #[getset(get = "pub")]
    storage_path: PathBuf,
    accounts: AccountStore,
    stop_polling_sender: Option<BroadcastSender<()>>,
    polling_handle: Option<thread::JoinHandle<()>>,
    generated_mnemonic: Option<String>,
    encrypted_accounts: Vec<String>,
}

impl Clone for AccountManager {
    /// Note that when cloning an AccountManager, the original reference's Drop will stop the background sync.
    /// When the cloned reference is dropped, the background sync system won't be stopped.
    fn clone(&self) -> Self {
        Self {
            storage_folder: self.storage_folder.clone(),
            storage_path: self.storage_path.clone(),
            accounts: self.accounts.clone(),
            stop_polling_sender: self.stop_polling_sender.clone(),
            polling_handle: None,
            generated_mnemonic: self.generated_mnemonic.clone(),
            encrypted_accounts: self.encrypted_accounts.clone(),
        }
    }
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        self.stop_background_sync();
    }
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    async fn load_accounts(storage_file_path: &PathBuf) -> crate::Result<(AccountStore, Vec<String>)> {
        let mut encrypted_accounts = Vec::new();
        let mut parsed_accounts = HashMap::new();

        let accounts = crate::storage::get(&storage_file_path)
            .await?
            .lock()
            .await
            .get_all()
            .await?;
        for parsed_account in accounts {
            match parsed_account {
                crate::storage::ParsedAccount::Account(account) => {
                    parsed_accounts.insert(account.id().clone(), account.into());
                }
                crate::storage::ParsedAccount::EncryptedAccount(value) => {
                    encrypted_accounts.push(value);
                }
            }
        }

        Ok((Arc::new(RwLock::new(parsed_accounts)), encrypted_accounts))
    }

    // error out if the storage is encrypted
    fn check_storage_encryption(&self) -> crate::Result<()> {
        if self.encrypted_accounts.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::StorageIsEncrypted)
        }
    }

    /// Starts monitoring the accounts with the node's mqtt topics.
    async fn start_monitoring(&self) -> crate::Result<()> {
        for account in self.accounts.read().await.values() {
            crate::monitor::monitor_account_addresses_balance(account.clone()).await?;
            crate::monitor::monitor_unconfirmed_messages(account.clone()).await?;
        }
        Ok(())
    }

    /// Initialises the background polling and MQTT monitoring.
    async fn start_background_sync(&mut self, polling_interval: Duration) {
        let monitoring_disabled = self.start_monitoring().await.is_err();
        let (stop_polling_sender, stop_polling_receiver) = broadcast_channel(1);
        self.start_polling(polling_interval, monitoring_disabled, stop_polling_receiver);
        self.stop_polling_sender = Some(stop_polling_sender);
    }

    /// Stops the background polling and MQTT monitoring.
    pub fn stop_background_sync(&mut self) {
        if let Some(polling_handle) = self.polling_handle.take() {
            self.stop_polling_sender
                .take()
                .unwrap()
                .send(())
                .expect("failed to stop polling process");
            polling_handle.join().expect("failed to join polling thread");
            let accounts = self.accounts.clone();
            thread::spawn(move || {
                crate::block_on(async move {
                    for account_handle in accounts.read().await.values() {
                        let _ = crate::monitor::unsubscribe(account_handle.clone());
                    }
                });
            })
            .join()
            .expect("failed to stop monitoring and polling systems");
        }
    }

    /// Sets the password for the stored accounts.
    pub async fn set_storage_password<P: AsRef<str>>(&mut self, password: P) -> crate::Result<()> {
        let key = storage_password_to_encryption_key(password.as_ref());
        // safe to unwrap because the storage is always defined at this point
        crate::storage::set_encryption_key(&self.storage_path, key)
            .await
            .unwrap();

        let mut accounts = self.accounts.write().await;
        for encrypted_account in &self.encrypted_accounts {
            let decrypted = crate::storage::decrypt_account_json(encrypted_account, &key)?;
            let account = serde_json::from_str::<Account>(&decrypted)?;
            accounts.insert(account.id().into(), account.into());
        }
        self.encrypted_accounts.clear();

        Ok(())
    }

    /// Sets the stronghold password.
    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "sqlite-storage", feature = "stronghold-storage"))))]
    pub async fn set_stronghold_password<P: AsRef<str>>(&mut self, password: P) -> crate::Result<()> {
        let mut dk = [0; 64];
        // safe to unwrap because rounds > 0
        crypto::kdfs::pbkdf::PBKDF2_HMAC_SHA512(password.as_ref().as_bytes(), b"wallet.rs", 100, &mut dk).unwrap();

        let stronghold_path = if self.storage_path.extension().unwrap_or_default() == "stronghold" {
            self.storage_path.clone()
        } else {
            self.storage_folder.join(STRONGHOLD_FILENAME)
        };
        crate::stronghold::load_snapshot(&stronghold_path, &dk[0..32][..].try_into().unwrap()).await?;

        // let is_empty = self.accounts.read().await.is_empty();
        if self.accounts.read().await.is_empty() {
            let (accounts, encrypted_accounts) = Self::load_accounts(&self.storage_path).await?;
            self.encrypted_accounts = encrypted_accounts;
            let mut accounts_store = self.accounts.write().await;
            for (id, account) in &*accounts.read().await {
                accounts_store.insert(id.clone(), account.clone());
            }
        }

        Ok(())
    }

    /// Starts the polling mechanism.
    fn start_polling(
        &mut self,
        polling_interval: Duration,
        is_monitoring_disabled: bool,
        mut stop: BroadcastReceiver<()>,
    ) {
        let storage_file_path = self.storage_path.clone();
        let accounts = self.accounts.clone();

        let handle = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async {
                let mut interval = interval(polling_interval);
                loop {
                    tokio::select! {
                        _ = async {
                            interval.tick().await;

                            let storage_file_path_ = storage_file_path.clone();

                            if let Err(error) = AssertUnwindSafe(poll(accounts.clone(), storage_file_path_, is_monitoring_disabled))
                                .catch_unwind()
                                .await {
                                    // if the error isn't a crate::Error type
                                if error.downcast_ref::<crate::Error>().is_none() {
                                    let msg = if let Some(message) = error.downcast_ref::<String>() {
                                        format!("Internal error: {}", message)
                                    } else if let Some(message) = error.downcast_ref::<&str>() {
                                        format!("Internal error: {}", message)
                                    } else {
                                        "Internal error".to_string()
                                    };
                                    log::error!("[POLLING] error: {}", msg);
                                    let _error = crate::Error::Panic(msg);
                                    // when the error is dropped, the on_error event will be triggered
                                }
                            }
                        } => {}
                        _ = stop.recv() => {
                            break;
                        }
                    }
                }
            });
        });
        self.polling_handle = Some(handle);
    }

    /// Stores a mnemonic for the given signer type.
    /// If the mnemonic is not provided, we'll generate one.
    pub async fn store_mnemonic(&mut self, signer_type: SignerType, mnemonic: Option<String>) -> crate::Result<()> {
        let mnemonic = match mnemonic {
            Some(m) => {
                self.verify_mnemonic(&m)?;
                m
            }
            None => self.generate_mnemonic()?,
        };

        let signer = crate::signing::get_signer(&signer_type).await;
        let mut signer = signer.lock().await;
        signer.store_mnemonic(&self.storage_path, mnemonic).await?;

        self.generated_mnemonic = None;

        Ok(())
    }

    /// Generates a new mnemonic.
    pub fn generate_mnemonic(&mut self) -> crate::Result<String> {
        let mut entropy = [0u8; 32];
        crypto::rand::fill(&mut entropy).map_err(|e| crate::Error::MnemonicEncode(format!("{:?}", e)))?;
        let mnemonic = crypto::bip39::wordlist::encode(&entropy, &crypto::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::MnemonicEncode(format!("{:?}", e)))?;
        self.generated_mnemonic = Some(mnemonic.clone());
        Ok(mnemonic)
    }

    /// Checks is the mnemonic is valid. If a mnemonic was generated with `generate_mnemonic()`, the mnemonic here
    /// should match the generated.
    pub fn verify_mnemonic<S: AsRef<str>>(&mut self, mnemonic: S) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::bip39::wordlist::verify(mnemonic.as_ref(), &crypto::bip39::wordlist::ENGLISH)
            // TODO: crypto::bip39::wordlist::Error should impl Display
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;

        // then we check if the provided mnemonic matches the mnemonic generated with `generate_mnemonic`
        if let Some(generated_mnemonic) = &self.generated_mnemonic {
            if generated_mnemonic != mnemonic.as_ref() {
                return Err(crate::Error::InvalidMnemonic(
                    "doesn't match the generated mnemonic".to_string(),
                ));
            }
            self.generated_mnemonic = None;
        }
        Ok(())
    }

    /// Adds a new account.
    pub fn create_account(&self, client_options: ClientOptions) -> crate::Result<AccountInitialiser> {
        self.check_storage_encryption()?;
        Ok(AccountInitialiser::new(
            client_options,
            self.accounts.clone(),
            self.storage_path.clone(),
        ))
    }

    /// Deletes an account.
    pub async fn remove_account<I: Into<AccountIdentifier>>(&self, account_id: I) -> crate::Result<()> {
        self.check_storage_encryption()?;

        let account_id = {
            let account_handle = self.get_account(account_id).await?;
            let account = account_handle.read().await;

            if !(account.messages().is_empty() && account.total_balance() == 0) {
                return Err(crate::Error::AccountNotEmpty);
            }

            account.id().to_string()
        };

        self.accounts.write().await.remove(&account_id);

        crate::storage::get(&self.storage_path)
            .await?
            .lock()
            .await
            .remove(&account_id)
            .await?;

        Ok(())
    }

    /// Syncs all accounts.
    pub async fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
        self.check_storage_encryption()?;

        sync_accounts(self.accounts.clone(), &self.storage_path, None).await
    }

    /// Transfers an amount from an account to another.
    pub async fn internal_transfer<F: Into<AccountIdentifier>, T: Into<AccountIdentifier>>(
        &self,
        from_account_id: F,
        to_account_id: T,
        amount: NonZeroU64,
    ) -> crate::Result<Message> {
        self.check_storage_encryption()?;

        let to_address = self
            .get_account(to_account_id)
            .await?
            .read()
            .await
            .latest_address()
            .ok_or(crate::Error::InternalTransferDestinationEmpty)?
            .clone();

        let from_synchronized = self.get_account(from_account_id).await?.sync().await.execute().await?;
        from_synchronized
            .transfer(Transfer::builder(to_address.address().clone(), amount).finish())
            .await
    }

    /// Backups the storage to the given destination
    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "sqlite-storage", feature = "stronghold-storage"))))]
    pub async fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<PathBuf> {
        let destination = destination.as_ref().to_path_buf();
        if !(destination.is_dir() && destination.exists()) {
            return Err(crate::Error::InvalidBackupDestination);
        }

        #[allow(unused_variables)]
        let (storage_path, backup_entire_directory) = (
            &self.storage_path,
            cfg!(feature = "stronghold") && cfg!(feature = "sqlite-storage"),
        );

        // if we're using SQLite for storage and stronghold for seed,
        // we'll backup only the stronghold file, copying SQLite data to its snapshot
        #[cfg(all(
            feature = "sqlite-storage",
            any(feature = "stronghold", feature = "stronghold-storage")
        ))]
        let (storage_path, backup_entire_directory) = {
            let storage_id = crate::storage::get(&&self.storage_path).await?.lock().await.id();
            // if we're actually using the SQLite storage adapter
            let storage_path = if storage_id == crate::storage::sqlite::STORAGE_ID {
                // create a account manager to setup the stronghold storage for the backup
                let _ = Self::builder()
                    .with_storage(
                        &self.storage_folder.join(STRONGHOLD_FILENAME),
                        ManagerStorage::Stronghold,
                        None,
                    )
                    .unwrap() // safe to unwrap - password is None
                    .with_storage_encryption_key(
                        crate::storage::get(&self.storage_path)
                            .await?
                            .lock()
                            .await
                            .encryption_key,
                    )
                    .skip_polling()
                    .finish()
                    .await?;
                let stronghold_storage = crate::storage::get(&self.storage_folder.join(STRONGHOLD_FILENAME)).await?;
                let mut stronghold_storage = stronghold_storage.lock().await;

                for account_handle in self.accounts.read().await.values() {
                    stronghold_storage
                        .set(
                            &account_handle.read().await.id(),
                            serde_json::to_string(&*account_handle.read().await)?,
                        )
                        .await?;
                }
                self.storage_folder.join(STRONGHOLD_FILENAME)
            } else {
                self.storage_path.clone()
            };
            (storage_path, false)
        };

        if storage_path.exists() {
            let destination = if backup_entire_directory {
                backup_dir(&self.storage_folder, &destination)?;
                destination
            } else if let Some(filename) = storage_path.file_name() {
                let destination = destination.join(backup_filename(filename.to_str().unwrap()));
                let res = fs::copy(storage_path, &destination);

                // if we're using SQLite for storage and stronghold for seed,
                // we'll remove the accounts from stronghold after the backup
                #[cfg(all(
                    feature = "sqlite-storage",
                    any(feature = "stronghold", feature = "stronghold-storage")
                ))]
                {
                    let storage_id = crate::storage::get(&self.storage_path).await?.lock().await.id();
                    // if we're actually using the SQLite storage adapter
                    if storage_id == crate::storage::sqlite::STORAGE_ID {
                        let mut stronghold_storage = crate::storage::stronghold::StrongholdStorageAdapter::new(
                            &self.storage_folder.join(STRONGHOLD_FILENAME),
                        )
                        .unwrap();
                        for account_handle in self.accounts.read().await.values() {
                            stronghold_storage.remove(&account_handle.read().await.id()).await?;
                        }
                    }
                }

                res?;
                destination
            } else {
                return Err(crate::Error::StorageDoesntExist);
            };
            Ok(destination)
        } else {
            Err(crate::Error::StorageDoesntExist)
        }
    }

    /// Import backed up accounts.
    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "sqlite-storage", feature = "stronghold-storage"))))]
    pub async fn import_accounts<S: AsRef<Path>>(
        &mut self,
        source: S,
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))] stronghold_password: impl AsRef<str>,
    ) -> crate::Result<()> {
        let source = source.as_ref();
        if source.is_dir() || !source.exists() {
            return Err(crate::Error::InvalidBackupFile);
        }

        #[allow(unused_variables)]
        #[cfg(feature = "stronghold-storage")]
        let storage_file_path = {
            let storage_file_path = self.storage_folder.join(STRONGHOLD_FILENAME);
            let storage_id = crate::storage::get(&self.storage_path).await?.lock().await.id();
            if storage_id == crate::storage::stronghold::STORAGE_ID && storage_file_path.exists() {
                return Err(crate::Error::StorageExists);
            }
            storage_file_path
        };
        #[cfg(feature = "sqlite-storage")]
        let storage_file_path = {
            if !self.accounts.read().await.is_empty() {
                return Err(crate::Error::StorageExists);
            }

            self.storage_folder.join(SQLITE_FILENAME)
        };

        fs::create_dir_all(&self.storage_folder)?;

        if cfg!(feature = "sqlite-storage") && source.extension().unwrap_or_default() == "stronghold" {
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            {
                let mut stronghold_manager = Self::builder()
                    .with_storage(&source, ManagerStorage::Stronghold, None)
                    .unwrap() // safe to unwrap - password is None
                    .with_storage_encryption_key(
                        crate::storage::get(&self.storage_path)
                            .await?
                            .lock()
                            .await
                            .encryption_key,
                    )
                    .skip_polling()
                    .finish()
                    .await?;
                stronghold_manager.set_stronghold_password(&stronghold_password).await?;
                for account_handle in stronghold_manager.accounts.read().await.values() {
                    account_handle.write().await.set_storage_path(self.storage_path.clone());
                }
                self.accounts = stronghold_manager.accounts.clone();
            }
            #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
            return Err(crate::Error::InvalidBackupFile);
        } else {
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            {
                // wait for stronghold to finish its tasks
                let _ = crate::stronghold::actor_runtime().lock().await;
            }
            fs::copy(source, &storage_file_path)?;
        }

        // the accounts map isn't empty when restoring SQLite from a stronghold snapshot
        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        if self.accounts.read().await.is_empty() {
            let (accounts, encrypted_accounts) = Self::load_accounts(&self.storage_path).await?;
            self.accounts = accounts;
            self.encrypted_accounts = encrypted_accounts;
        }

        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        {
            // force stronghold to read the snapshot again, ignoring any previous cached value
            crate::stronghold::unload_snapshot(&self.storage_path, false).await?;
            if let Err(e) = self.set_stronghold_password(stronghold_password).await {
                fs::remove_file(&storage_file_path)?;
                return Err(e);
            }
        }

        Ok(())
    }

    /// Gets the account associated with the given identifier.
    pub async fn get_account<I: Into<AccountIdentifier>>(&self, account_id: I) -> crate::Result<AccountHandle> {
        self.check_storage_encryption()?;
        let account_id = account_id.into();
        let accounts = self.accounts.read().await;

        let account = match account_id {
            AccountIdentifier::Id(id) => accounts.get(&id),
            AccountIdentifier::Index(index) => {
                let mut associated_account = None;
                for account_handle in accounts.values() {
                    let account = account_handle.read().await;
                    if account.index() == &index {
                        // if we already found an account with this index,
                        // we error out since this is an incorrect usage of the API
                        // you can't use the index to get an account if you're using multiple signer types
                        // since there's multiple index sequences in that case
                        if associated_account.is_some() {
                            return Err(crate::Error::CannotUseIndexIdentifier);
                        }
                        associated_account = Some(account_handle);
                    }
                }
                associated_account
            }
            AccountIdentifier::Alias(alias) => {
                let mut associated_account = None;
                for account_handle in accounts.values() {
                    let account = account_handle.read().await;
                    if account.alias() == &alias {
                        associated_account = Some(account_handle);
                        break;
                    }
                }
                associated_account
            }
            AccountIdentifier::Address(address) => {
                let mut associated_account = None;
                for account_handle in accounts.values() {
                    let account = account_handle.read().await;
                    if account.addresses().iter().any(|a| a.address() == &address) {
                        associated_account = Some(account_handle);
                        break;
                    }
                }
                associated_account
            }
        };

        account.cloned().ok_or(crate::Error::AccountNotFound)
    }

    /// Gets all accounts from storage.
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        self.check_storage_encryption()?;
        let accounts = self.accounts.read().await;
        Ok(accounts.values().cloned().collect())
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.reattach(message_id).await
    }

    /// Promotes an unconfirmed transaction.
    pub async fn promote<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.promote(message_id).await
    }

    /// Retries an unconfirmed transaction.
    pub async fn retry<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.retry(message_id).await
    }
}

async fn poll(accounts: AccountStore, storage_file_path: PathBuf, syncing: bool) -> crate::Result<()> {
    let retried = if syncing {
        let mut accounts_before_sync = Vec::new();
        for account_handle in accounts.read().await.values() {
            accounts_before_sync.push(account_handle.read().await.clone());
        }
        let synced_accounts = sync_accounts(accounts.clone(), &storage_file_path, Some(0)).await?;
        let accounts_after_sync = accounts.read().await;

        log::debug!("[POLLING] synced accounts");

        // compare accounts to check for balance changes and new messages
        for account_before_sync in &accounts_before_sync {
            let account_after_sync = accounts_after_sync.get(account_before_sync.id()).unwrap();
            let account_after_sync = account_after_sync.read().await;

            // balance event
            for address_before_sync in account_before_sync.addresses() {
                let address_after_sync = account_after_sync
                    .addresses()
                    .iter()
                    .find(|addr| addr == &address_before_sync)
                    .unwrap();
                if address_after_sync.balance() != address_before_sync.balance() {
                    log::debug!(
                        "[POLLING] address {} balance changed from {} to {}",
                        address_after_sync.address().to_bech32(),
                        address_before_sync.balance(),
                        address_after_sync.balance()
                    );
                    emit_balance_change(
                        account_after_sync.id(),
                        address_after_sync,
                        *address_after_sync.balance(),
                    );
                }
            }

            // new messages event
            account_after_sync
                .messages()
                .iter()
                .filter(|message| !account_before_sync.messages().contains(message))
                .for_each(|message| {
                    log::info!("[POLLING] new message: {:?}", message.id());
                    emit_transaction_event(TransactionEventType::NewTransaction, account_after_sync.id(), &message)
                });

            // confirmation state change event
            for message in account_after_sync.messages() {
                let changed = match account_before_sync.messages().iter().find(|m| m.id() == message.id()) {
                    Some(old_message) => message.confirmed() != old_message.confirmed(),
                    None => false,
                };
                if changed {
                    log::info!("[POLLING] message confirmed: {:?}", message.id());
                    emit_confirmation_state_change(account_after_sync.id(), &message, true);
                }
            }
        }
        retry_unconfirmed_transactions(synced_accounts).await?
    } else {
        log::info!("[POLLING] skipping syncing process because MQTT is running");
        let mut retried_messages = vec![];
        for account_handle in accounts.read().await.values() {
            let (account_id, unconfirmed_messages): (String, Vec<(MessageId, Payload)>) = {
                let account = account_handle.read().await;
                let account_id = account.id().clone();
                let unconfirmed_messages = account
                    .list_messages(account.messages().len(), 0, Some(MessageType::Unconfirmed))
                    .iter()
                    .map(|m| (*m.id(), m.payload().clone()))
                    .collect();
                (account_id, unconfirmed_messages)
            };

            let mut promotions = vec![];
            let mut reattachments = vec![];
            for (message_id, payload) in unconfirmed_messages {
                let new_message = repost_message(account_handle.clone(), &message_id, RepostAction::Retry).await?;
                if new_message.payload() == &payload {
                    reattachments.push(new_message);
                } else {
                    log::info!("[POLLING] promoted and new message is {:?}", new_message.id());
                    promotions.push(new_message);
                }
            }

            retried_messages.push(RetriedData {
                promoted: promotions,
                reattached: reattachments,
                account_id,
            });
        }

        retried_messages
    };

    retried.iter().for_each(|retried_data| {
        retried_data.reattached.iter().for_each(|message| {
            emit_transaction_event(TransactionEventType::Reattachment, &retried_data.account_id, &message);
        });
    });
    Ok(())
}

async fn discover_accounts(
    accounts: AccountStore,
    storage_path: &PathBuf,
    client_options: &ClientOptions,
    signer_type: Option<SignerType>,
) -> crate::Result<Vec<(AccountHandle, SyncedAccount)>> {
    let mut synced_accounts = vec![];
    loop {
        let mut account_initialiser =
            AccountInitialiser::new(client_options.clone(), accounts.clone(), storage_path.clone()).skip_persistance();
        if let Some(signer_type) = &signer_type {
            account_initialiser = account_initialiser.signer_type(signer_type.clone());
        }
        let account_handle = account_initialiser.initialise().await?;
        log::debug!(
            "[SYNC] discovering account {}, signer type {:?}",
            account_handle.read().await.alias(),
            account_handle.read().await.signer_type()
        );
        let synced_account = account_handle.sync().await.execute().await?;
        let is_empty = *synced_account.is_empty();
        log::debug!("[SYNC] account is empty? {}", is_empty);
        if is_empty {
            break;
        } else {
            synced_accounts.push((account_handle, synced_account));
        }
    }
    Ok(synced_accounts)
}

async fn sync_accounts<'a>(
    accounts: AccountStore,
    storage_file_path: &PathBuf,
    address_index: Option<usize>,
) -> crate::Result<Vec<SyncedAccount>> {
    let mut synced_accounts = vec![];
    let mut last_account = None;

    {
        let accounts = accounts.read().await;
        for account_handle in accounts.values() {
            let mut sync = account_handle.sync().await;
            if let Some(index) = address_index {
                sync = sync.address_index(index);
            }
            let synced_account = sync.execute().await?;

            let account = account_handle.read().await;
            last_account = Some((
                account.messages().is_empty() || account.addresses().iter().all(|addr| *addr.balance() == 0),
                account.client_options().clone(),
                account.signer_type().clone(),
            ));
            synced_accounts.push(synced_account);
        }
    }

    let discovered_accounts_res = match last_account {
        Some((is_empty, client_options, signer_type)) => {
            if is_empty {
                log::debug!("[SYNC] running account discovery because the latest account is empty");
                discover_accounts(accounts.clone(), &storage_file_path, &client_options, Some(signer_type)).await
            } else {
                log::debug!("[SYNC] skipping account discovery because the latest account isn't empty");
                Ok(vec![])
            }
        }
        None => Ok(vec![]), /* None => discover_accounts(accounts.clone(), &storage_path, &ClientOptions::default(),
                             * None).await, */
    };

    if let Ok(discovered_accounts) = discovered_accounts_res {
        if !discovered_accounts.is_empty() {
            let mut accounts = accounts.write().await;
            for (account_handle, synced_account) in discovered_accounts {
                account_handle.write().await.set_skip_persistance(false);
                accounts.insert(account_handle.id().await, account_handle);
                synced_accounts.push(synced_account);
            }
        }
    }

    Ok(synced_accounts)
}

struct RetriedData {
    #[allow(dead_code)]
    promoted: Vec<Message>,
    reattached: Vec<Message>,
    account_id: String,
}

async fn retry_unconfirmed_transactions(synced_accounts: Vec<SyncedAccount>) -> crate::Result<Vec<RetriedData>> {
    let mut retried_messages = vec![];
    for synced in synced_accounts {
        let account = synced.account_handle().read().await;

        let unconfirmed_messages = account.list_messages(account.messages().len(), 0, Some(MessageType::Unconfirmed));
        let mut reattachments = vec![];
        let mut promotions = vec![];
        for message in unconfirmed_messages {
            log::debug!("[POLLING] retrying {:?}", message);
            let new_message = synced.retry(message.id()).await?;
            // if the payload is the same, it was reattached; otherwise it was promoted
            if new_message.payload() == message.payload() {
                log::debug!("[POLLING] rettached and new message is {:?}", new_message);
                reattachments.push(new_message);
            } else {
                log::debug!("[POLLING] promoted and new message is {:?}", new_message);
                promotions.push(new_message);
            }
        }
        retried_messages.push(RetriedData {
            promoted: promotions,
            reattached: reattachments,
            account_id: account.id().clone(),
        });
    }
    Ok(retried_messages)
}

fn backup_dir<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        let src: PathBuf = working_path.components().skip(input_root).collect();

        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                let dest_path = dest.join(backup_filename(filename.to_str().unwrap()));
                fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}

fn backup_filename(original: &str) -> String {
    let date = Local::now();
    format!(
        "{}-iota-wallet-backup{}",
        date.format("%FT%H-%M-%S").to_string(),
        if original.is_empty() {
            "".to_string()
        } else {
            format!("-{}", original)
        }
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        address::{AddressBuilder, AddressWrapper, IotaAddress},
        client::ClientOptionsBuilder,
        message::Message,
    };
    use iota::{Ed25519Address, IndexationPayload, MessageBuilder, MessageId, Payload};

    #[tokio::test]
    async fn store_accounts() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let account_handle = manager
            .create_account(client_options)
            .unwrap()
            .alias("alias")
            .initialise()
            .await
            .expect("failed to add account");

        manager
            .remove_account(account_handle.read().await.id())
            .await
            .expect("failed to remove account");
    }

    #[tokio::test]
    async fn duplicated_alias() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();
        let alias = "alias";

        manager
            .create_account(client_options.clone())
            .unwrap()
            .alias(alias)
            .initialise()
            .await
            .expect("failed to add account");

        let second_create_response = manager
            .create_account(client_options)
            .unwrap()
            .alias(alias)
            .initialise()
            .await;
        assert_eq!(second_create_response.is_err(), true);
        match second_create_response.unwrap_err() {
            crate::Error::AccountAliasAlreadyExists => {}
            _ => panic!("unexpected create account response; expected AccountAliasAlreadyExists"),
        }
    }

    #[tokio::test]
    async fn get_account() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let account_handle1 = manager
            .create_account(client_options.clone())
            .unwrap()
            .alias("alias")
            .initialise()
            .await
            .expect("failed to add account");
        account_handle1.generate_address().await.unwrap();
        {
            // update address balance so we can create the next account
            let mut account = account_handle1.write().await;
            for address in account.addresses_mut() {
                address.set_balance(5);
            }
        }

        let account_handle2 = manager
            .create_account(client_options)
            .unwrap()
            .alias("alias2")
            .initialise()
            .await
            .expect("failed to add account");
        account_handle2.generate_address().await.unwrap();

        let account1 = &*account_handle1.read().await;
        let account2 = &*account_handle2.read().await;

        assert_eq!(
            account1,
            &*manager.get_account(*account1.index()).await.unwrap().read().await
        );
        assert_eq!(
            account1,
            &*manager.get_account(account1.alias()).await.unwrap().read().await
        );
        assert_eq!(
            account1,
            &*manager.get_account(account1.id()).await.unwrap().read().await
        );
        assert_eq!(
            account1,
            &*manager
                .get_account(account1.addresses().first().unwrap().address().to_bech32())
                .await
                .unwrap()
                .read()
                .await
        );

        assert_eq!(
            account2,
            &*manager.get_account(*account2.index()).await.unwrap().read().await
        );
        assert_eq!(
            account2,
            &*manager.get_account(account2.alias()).await.unwrap().read().await
        );
        assert_eq!(
            account2,
            &*manager.get_account(account2.id()).await.unwrap().read().await
        );
        assert_eq!(
            account2,
            &*manager
                .get_account(account2.addresses().first().unwrap().address().to_bech32())
                .await
                .unwrap()
                .read()
                .await
        );
    }

    #[tokio::test]
    async fn remove_account_with_message_history() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let messages = vec![Message::from_iota_message(
            MessageId::new([0; 32]),
            &[],
            &MessageBuilder::<crate::test_utils::NoopNonceProvider>::new()
                .with_parent1(MessageId::new([0; 32]))
                .with_parent2(MessageId::new([0; 32]))
                .with_payload(Payload::Indexation(Box::new(
                    IndexationPayload::new("index".to_string(), &[0; 16]).unwrap(),
                )))
                .with_network_id(0)
                .with_nonce_provider(crate::test_utils::NoopNonceProvider {}, 0f64)
                .finish()
                .unwrap(),
            None,
        )
        .unwrap()];

        let account_handle = manager
            .create_account(client_options)
            .unwrap()
            .messages(messages)
            .initialise()
            .await
            .unwrap();

        let account = account_handle.read().await;
        let remove_response = manager.remove_account(account.id()).await;
        assert!(remove_response.is_err());
    }

    #[tokio::test]
    async fn remove_account_with_balance() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let account_handle = manager
            .create_account(client_options)
            .unwrap()
            .addresses(vec![AddressBuilder::new()
                .balance(5)
                .key_index(0)
                .address(AddressWrapper::new(
                    IotaAddress::Ed25519(Ed25519Address::new([0; 32])),
                    "iota".to_string(),
                ))
                .outputs(vec![])
                .build()
                .unwrap()])
            .initialise()
            .await
            .unwrap();
        let account = account_handle.read().await;

        let remove_response = manager.remove_account(account.id()).await;
        assert!(remove_response.is_err());
    }

    #[tokio::test]
    async fn create_account_with_latest_without_history() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        manager
            .create_account(client_options.clone())
            .unwrap()
            .alias("alias")
            .initialise()
            .await
            .expect("failed to add account");

        let create_response = manager.create_account(client_options).unwrap().initialise().await;
        assert!(create_response.is_err());
    }

    #[tokio::test]
    async fn backup_and_restore_happy_path() {
        let backup_path = "./backup/happy-path";
        let _ = std::fs::remove_dir_all(backup_path);
        std::fs::create_dir_all(backup_path).unwrap();

        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let account_handle = manager
            .create_account(client_options)
            .unwrap()
            .alias("alias")
            .signer_type(crate::test_utils::signer_type())
            .initialise()
            .await
            .expect("failed to add account");

        // backup the stored accounts to ./backup/happy-path/${backup_name}
        let backup_path = manager.backup(backup_path).await.unwrap();
        let backup_file_path = if backup_path.is_dir() {
            std::fs::read_dir(backup_path)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path()
                .to_path_buf()
        } else {
            backup_path
        };

        // get another manager instance so we can import the accounts to a different storage
        #[allow(unused_mut)]
        let mut manager = crate::test_utils::get_account_manager().await;

        #[cfg(all(
            not(feature = "sqlite-storage"),
            any(feature = "stronghold", feature = "stronghold-storage")
        ))]
        {
            // wait for stronghold to finish pending operations and delete the storage file
            crate::stronghold::unload_snapshot(manager.storage_path(), false)
                .await
                .unwrap();
            let _ = crate::stronghold::actor_runtime().lock().await;
            std::fs::remove_file(manager.storage_path()).unwrap();
        }

        manager.set_storage_password("password").await.unwrap();

        // import the accounts from the backup and assert that it's the same

        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        manager.import_accounts(backup_file_path, "password").await.unwrap();

        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        manager.import_accounts(backup_file_path).await.unwrap();

        let imported_account = manager.get_account(account_handle.read().await.alias()).await.unwrap();
        // set the account storage path field so the assert works
        account_handle
            .write()
            .await
            .set_storage_path(manager.storage_path().clone());
        assert_eq!(&*account_handle.read().await, &*imported_account.read().await);
    }
}

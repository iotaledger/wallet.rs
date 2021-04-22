// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        AccountHandle, AccountIdentifier, AccountInitialiser, AccountSynchronizeStep, AccountSynchronizer,
        SyncedAccount, SyncedAccountData,
    },
    address::{AddressOutput, AddressWrapper},
    client::ClientOptions,
    event::{
        emit_balance_change, emit_confirmation_state_change, emit_reattachment_event, emit_transaction_event,
        BalanceEvent, TransactionConfirmationChangeEvent, TransactionEvent, TransactionEventType,
        TransactionReattachmentEvent,
    },
    message::{Message, MessagePayload, MessageType, TransactionEssence, Transfer},
    signing::SignerType,
    storage::{StorageAdapter, Timestamp},
};

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::TryInto,
    fs,
    hash::{Hash, Hasher},
    num::NonZeroU64,
    ops::{Deref, Range},
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use chrono::prelude::*;
use futures::FutureExt;
use getset::Getters;
use iota::{bee_rest_api::types::dtos::LedgerInclusionStateDto, MessageId, OutputId};
use serde::Serialize;
use tokio::{
    sync::{
        broadcast::{channel as broadcast_channel, Receiver as BroadcastReceiver, Sender as BroadcastSender},
        Mutex, RwLock,
    },
    time::interval,
};
use zeroize::Zeroize;

mod migration;
pub use migration::*;

/// The default storage folder.
pub const DEFAULT_STORAGE_FOLDER: &str = "./storage";

const DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD: usize = 100;

/// The default stronghold storage file name.
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub const STRONGHOLD_FILENAME: &str = "wallet.stronghold";

/// The default RocksDB storage path.
pub const ROCKSDB_FILENAME: &str = "db";

type AccountsMap = HashMap<String, AccountHandle>;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct AccountStore(Arc<RwLock<AccountsMap>>);

impl AccountStore {
    pub(crate) fn new(inner: Arc<RwLock<AccountsMap>>) -> Self {
        Self(inner)
    }
}

impl Deref for AccountStore {
    type Target = RwLock<AccountsMap>;
    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}

/// The storage used by the manager.
enum ManagerStorage {
    /// Stronghold storage.
    Stronghold,
    /// RocksDB storage.
    Rocksdb,
}

fn storage_password_to_encryption_key(password: &str) -> [u8; 32] {
    let mut dk = [0; 64];
    // safe to unwrap (rounds > 0)
    crypto::keys::pbkdf::PBKDF2_HMAC_SHA512(password.as_bytes(), b"wallet.rs::storage", 100, &mut dk).unwrap();
    let key: [u8; 32] = dk[0..32][..].try_into().unwrap();
    key
}

/// Account manager builder.
pub struct AccountManagerBuilder {
    storage_folder: PathBuf,
    storage_file_name: Option<String>,
    storage: ManagerStorage,
    polling_interval: Duration,
    skip_polling: bool,
    storage_encryption_key: Option<[u8; 32]>,
    account_options: AccountOptions,
}

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        Self {
            storage_folder: PathBuf::from(DEFAULT_STORAGE_FOLDER),
            storage_file_name: None,
            storage: ManagerStorage::Rocksdb,
            polling_interval: Duration::from_millis(30_000),
            skip_polling: false,
            storage_encryption_key: None,
            account_options: AccountOptions {
                output_consolidation_threshold: DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
                automatic_output_consolidation: true,
                sync_spent_outputs: false,
                persist_events: false,
                allow_create_multiple_empty_accounts: false,
            },
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the storage config to be used.
    pub fn with_storage(mut self, storage_folder: impl AsRef<Path>, password: Option<&str>) -> crate::Result<Self> {
        self.storage_folder = storage_folder.as_ref().to_path_buf();
        self.storage_encryption_key = password.map(|p| storage_password_to_encryption_key(p));
        Ok(self)
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

    pub(crate) fn with_storage_file_name<F: Into<String>>(mut self, file_name: F) -> Self {
        self.storage_file_name.replace(file_name.into());
        self
    }

    /// Use stronghold as storage system.
    pub(crate) fn with_stronghold_storage(mut self) -> Self {
        self.storage = ManagerStorage::Stronghold;
        self
    }

    /// Sets the number of outputs an address must have to trigger the automatic consolidation process.
    pub fn with_output_consolidation_threshold(mut self, threshold: usize) -> Self {
        self.account_options.output_consolidation_threshold = threshold;
        self
    }

    /// Disables the automatic output consolidation process.
    pub fn with_automatic_output_consolidation_disabled(mut self) -> Self {
        self.account_options.automatic_output_consolidation = true;
        self
    }

    /// Enables fetching spent output history on sync.
    pub fn with_sync_spent_outputs(mut self) -> Self {
        self.account_options.sync_spent_outputs = true;
        self
    }

    /// Enables event persistence.
    pub fn with_event_persistence(mut self) -> Self {
        self.account_options.persist_events = true;
        self
    }

    /// Enables creating multiple accounts without history.
    /// The wallet disables it by default to simplify account discovery.
    pub fn with_multiple_empty_accounts(mut self) -> Self {
        self.account_options.allow_create_multiple_empty_accounts = true;
        self
    }

    /// Builds the manager.
    pub async fn finish(self) -> crate::Result<AccountManager> {
        let (storage, storage_file_path, is_stronghold): (
            Option<Box<dyn StorageAdapter + Send + Sync>>,
            PathBuf,
            bool,
        ) = match self.storage {
            ManagerStorage::Stronghold => {
                let path = self
                    .storage_folder
                    .join(self.storage_file_name.as_deref().unwrap_or(STRONGHOLD_FILENAME));
                fs::create_dir_all(&self.storage_folder)?;
                let storage = crate::storage::stronghold::StrongholdStorageAdapter::new(&path)?;
                (
                    Some(Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>),
                    path,
                    true,
                )
            }
            ManagerStorage::Rocksdb => {
                let path = self
                    .storage_folder
                    .join(self.storage_file_name.as_deref().unwrap_or(ROCKSDB_FILENAME));
                fs::create_dir_all(&self.storage_folder)?;
                // rocksdb storage already exists; no need to create a new instance
                let storage = if crate::storage::get(&path).await.is_ok() {
                    None
                } else {
                    let storage = crate::storage::rocksdb::RocksdbStorageAdapter::new(&path)?;
                    Some(Box::new(storage) as Box<dyn StorageAdapter + Send + Sync>)
                };
                (storage, path, false)
            }
        };

        if let Some(storage) = storage {
            crate::storage::set(&storage_file_path, self.storage_encryption_key, storage).await;
        }

        let sync_accounts_lock = Arc::new(Mutex::new(()));

        // with the stronghold storage, the accounts are loaded when the password is set
        let (accounts, loaded_accounts) = if is_stronghold {
            (AccountStore::new(Default::default()), false)
        } else {
            let accounts = AccountStore::new(Default::default());
            let res = AccountManager::load_accounts(
                &accounts,
                &storage_file_path,
                self.account_options,
                sync_accounts_lock.clone(),
            )
            .await;
            (accounts, res.is_ok())
        };
        let mut instance = AccountManager {
            storage_folder: self.storage_folder,
            loaded_accounts,
            storage_path: storage_file_path,
            accounts,
            stop_polling_sender: None,
            polling_handle: None,
            generated_mnemonic: None,
            account_options: self.account_options,
            sync_accounts_lock,
            cached_migration_data: Default::default(),
            cached_migration_bundles: Default::default(),
        };

        if !self.skip_polling {
            instance
                .start_background_sync(
                    self.polling_interval,
                    self.account_options.automatic_output_consolidation,
                )
                .await;
        }

        Ok(instance)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct AccountOptions {
    pub(crate) output_consolidation_threshold: usize,
    pub(crate) automatic_output_consolidation: bool,
    pub(crate) sync_spent_outputs: bool,
    pub(crate) persist_events: bool,
    pub(crate) allow_create_multiple_empty_accounts: bool,
}

#[derive(Clone)]
pub(crate) struct CachedMigrationData {
    nodes: Vec<String>,
    permanode: Option<String>,
    security_level: u8,
    inputs: HashMap<Range<u64>, Vec<InputData>>,
}

/// Created migration bundle data.
#[derive(Debug, Clone, Getters, Serialize)]
#[getset(get = "pub")]
pub struct MigrationBundle {
    /// The bundle crackability if it was mined.
    crackability: f64,
    /// The bundle hash.
    #[serde(rename = "bundleHash")]
    bundle_hash: String,
}

/// Response from `send_migration_bundle`.
#[derive(Debug, Getters, Serialize)]
#[getset(get = "pub")]
pub struct MigratedBundle {
    /// Hash of the tail transaction.
    #[serde(rename = "tailTransactionHash")]
    tail_transaction_hash: String,
    /// The deposit address.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The bundle input value.
    value: u64,
}

type CachedMigrationBundle = (Vec<BundledTransaction>, AddressWrapper, u64);

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Getters)]
pub struct AccountManager {
    storage_folder: PathBuf,
    loaded_accounts: bool,
    /// the path to the storage.
    #[getset(get = "pub")]
    storage_path: PathBuf,
    /// Returns a handle to the accounts store.
    #[getset(get = "pub")]
    accounts: AccountStore,
    stop_polling_sender: Option<BroadcastSender<()>>,
    polling_handle: Option<thread::JoinHandle<()>>,
    generated_mnemonic: Option<String>,
    account_options: AccountOptions,
    sync_accounts_lock: Arc<Mutex<()>>,
    cached_migration_data: Mutex<HashMap<u64, CachedMigrationData>>,
    cached_migration_bundles: Mutex<HashMap<String, CachedMigrationBundle>>,
}

impl Clone for AccountManager {
    /// Note that when cloning an AccountManager, the original reference's Drop will stop the background sync.
    /// When the cloned reference is dropped, the background sync system won't be stopped.
    ///
    /// Additionally, the generated mnemonic and migration data isn't cloned for security reasons.
    fn clone(&self) -> Self {
        Self {
            storage_folder: self.storage_folder.clone(),
            loaded_accounts: self.loaded_accounts,
            storage_path: self.storage_path.clone(),
            accounts: self.accounts.clone(),
            stop_polling_sender: self.stop_polling_sender.clone(),
            polling_handle: None,
            generated_mnemonic: None,
            account_options: self.account_options,
            sync_accounts_lock: self.sync_accounts_lock.clone(),
            cached_migration_data: Default::default(),
            cached_migration_bundles: Default::default(),
        }
    }
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        self.stop_background_sync();
    }
}

#[cfg(feature = "stronghold")]
fn stronghold_password<P: Into<String>>(password: P) -> Vec<u8> {
    let mut password = password.into();
    let mut dk = [0; 64];
    // safe to unwrap because rounds > 0
    crypto::keys::pbkdf::PBKDF2_HMAC_SHA512(password.as_bytes(), b"wallet.rs", 100, &mut dk).unwrap();
    password.zeroize();
    let password: [u8; 32] = dk[0..32][..].try_into().unwrap();
    password.to_vec()
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    /// Gets the legacy migration data for the seed.
    pub async fn get_migration_data(&self, finder: MigrationDataFinder<'_>) -> crate::Result<MigrationData> {
        if finder.initial_address_index == 0 {
            self.cached_migration_data.lock().await.remove(&finder.seed_hash);
        }
        let metadata = finder
            .finish(
                self.cached_migration_data
                    .lock()
                    .await
                    .get(&finder.seed_hash)
                    .map(|c| c.inputs.clone())
                    .unwrap_or_default(),
            )
            .await?;
        self.cached_migration_data
            .lock()
            .await
            .entry(finder.seed_hash)
            .or_insert(CachedMigrationData {
                nodes: finder.nodes.iter().map(|node| node.to_string()).collect(),
                permanode: finder.permanode.map(|node| node.to_string()),
                security_level: finder.security_level,
                inputs: Default::default(),
            })
            .inputs
            .extend(metadata.inputs.clone().into_iter());

        Ok(MigrationData {
            balance: metadata.balance,
            last_checked_address_index: metadata.last_checked_address_index,
            inputs: metadata.inputs.into_iter().map(|(_, v)| v).flatten().collect(),
        })
    }

    /// Creates the bundle for migration associated with the given input indexes,
    /// Performs bundle mining if the address is spent and `mine` is true,
    /// And signs the bundle. Returns the bundle hash.
    /// It logs the operations to `$storage_path.join(log_file_name)`.
    pub async fn create_migration_bundle(
        &self,
        seed: &str,
        input_address_indexes: &[u64],
        mine: bool,
        timeout: Duration,
        offset: i64,
        log_file_name: &str,
    ) -> crate::Result<MigrationBundle> {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed_hash = hasher.finish();

        let seed = TernarySeed::from_trits(TryteBuf::try_from_str(&seed).unwrap().as_trits().encode::<T1B1Buf>())
            .map_err(|_| crate::Error::InvalidSeed)?;
        let data = self
            .cached_migration_data
            .lock()
            .await
            .get(&seed_hash)
            .ok_or(crate::Error::MigrationDataNotFound)?
            .clone();

        let mut address_inputs: Vec<&InputData> = Default::default();
        let mut value = 0;
        for index in input_address_indexes {
            for inputs in data.inputs.values() {
                if let Some(input) = inputs.iter().find(|i| &i.index == index) {
                    value += input.balance;
                    address_inputs.push(input);
                    break;
                }
            }
        }

        let account_handle = self.get_account(0).await?;
        let bundle_data = migration::create_bundle(
            account_handle.clone(),
            &data,
            seed,
            address_inputs,
            mine,
            timeout,
            offset,
            self.storage_folder.join(log_file_name),
        )
        .await?;
        let crackability = bundle_data.crackability;
        let bundle_hash = bundle_data
            .bundle
            .first()
            .unwrap()
            .bundle()
            .to_inner()
            .encode::<T3B1Buf>()
            .iter_trytes()
            .map(char::from)
            .collect::<String>();
        let address = account_handle.latest_address().await.address().clone();
        self.cached_migration_bundles
            .lock()
            .await
            .insert(bundle_hash.clone(), (bundle_data.bundle, address, value));

        Ok(MigrationBundle {
            crackability,
            bundle_hash,
        })
    }

    /// Sends the migration bundle to the given node.
    pub async fn send_migration_bundle(&self, nodes: &[&str], hash: &str, mwm: u8) -> crate::Result<MigratedBundle> {
        let (bundle, address, value) = self
            .cached_migration_bundles
            .lock()
            .await
            .get(hash)
            .ok_or(crate::Error::MigrationBundleNotFound)?
            .clone();
        let tail_transaction_hash = migration::send_bundle(nodes, bundle.to_vec(), mwm).await?;
        self.cached_migration_bundles.lock().await.remove(hash);

        Ok(MigratedBundle {
            tail_transaction_hash: tail_transaction_hash
                .to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>(),
            address,
            value,
        })
    }

    async fn load_accounts(
        accounts: &AccountStore,
        storage_file_path: &Path,
        account_options: AccountOptions,
        sync_accounts_lock: Arc<Mutex<()>>,
    ) -> crate::Result<()> {
        let loaded_accounts = crate::storage::get(&storage_file_path)
            .await?
            .lock()
            .await
            .get_accounts()
            .await?;
        for account in loaded_accounts {
            accounts.write().await.insert(
                account.id().clone(),
                AccountHandle::new(account, accounts.clone(), account_options, sync_accounts_lock.clone()),
            );
        }

        Ok(())
    }

    /// Deletes the storage.
    pub async fn delete(self) -> Result<(), (crate::Error, Self)> {
        self.delete_internal().await.map_err(|e| (e, self))
    }

    pub(crate) async fn delete_internal(&self) -> crate::Result<()> {
        // safe to unwrap: we know the storage exists
        let storage_id = crate::storage::remove(&self.storage_path).await.unwrap();

        if self.storage_path.exists() {
            if self.storage_path.is_file() {
                std::fs::remove_file(&self.storage_path)?;
            } else {
                std::fs::remove_dir_all(&self.storage_path)?;
            }
        }

        #[cfg(feature = "stronghold")]
        {
            crate::stronghold::unload_snapshot(&self.storage_path, false).await?;
            std::fs::remove_file(self.stronghold_snapshot_path_internal(&storage_id).await?)?;
        }

        Ok(())
    }

    #[cfg(feature = "stronghold")]
    pub(crate) async fn stronghold_snapshot_path(&self) -> crate::Result<PathBuf> {
        let storage_id = crate::storage::get(&self.storage_path).await?.lock().await.id();
        self.stronghold_snapshot_path_internal(storage_id).await
    }

    #[cfg(feature = "stronghold")]
    pub(crate) async fn stronghold_snapshot_path_internal(&self, storage_id: &str) -> crate::Result<PathBuf> {
        let stronghold_snapshot_path = if storage_id == crate::storage::stronghold::STORAGE_ID {
            self.storage_path.clone()
        } else {
            self.storage_folder.join(STRONGHOLD_FILENAME)
        };
        Ok(stronghold_snapshot_path)
    }

    // error out if the storage is encrypted
    fn check_storage_encryption(&self) -> crate::Result<()> {
        if self.loaded_accounts {
            Ok(())
        } else {
            Err(crate::Error::StorageIsEncrypted)
        }
    }

    /// Starts monitoring the accounts with the node's mqtt topics.
    async fn start_monitoring(accounts: AccountStore) {
        for account in accounts.read().await.values() {
            crate::monitor::monitor_account_addresses_balance(account.clone()).await;
        }
    }

    /// Initialises the background polling and MQTT monitoring.
    async fn start_background_sync(&mut self, polling_interval: Duration, automatic_output_consolidation: bool) {
        Self::start_monitoring(self.accounts.clone()).await;
        let (stop_polling_sender, stop_polling_receiver) = broadcast_channel(1);
        self.start_polling(polling_interval, stop_polling_receiver, automatic_output_consolidation);
        self.stop_polling_sender.replace(stop_polling_sender);
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
                        let _ = crate::monitor::unsubscribe(account_handle.clone()).await;
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

        if self.accounts.read().await.is_empty() {
            Self::load_accounts(
                &self.accounts,
                &self.storage_path,
                self.account_options,
                self.sync_accounts_lock.clone(),
            )
            .await?;
            self.loaded_accounts = true;
            crate::spawn(Self::start_monitoring(self.accounts.clone()));
        } else {
            // save the accounts and messages again to reencrypt with the new key
            for account_handle in self.accounts.read().await.values() {
                let mut account = account_handle.write().await;
                account.save().await?;
                let messages = account.list_messages(0, 0, None).await?;
                account.save_messages(messages).await?;
            }
        }

        Ok(())
    }

    /// Sets the stronghold password.
    pub async fn set_stronghold_password<P: Into<String>>(&mut self, password: P) -> crate::Result<()> {
        let stronghold_path = if self.storage_path.extension().unwrap_or_default() == "stronghold" {
            self.storage_path.clone()
        } else {
            self.storage_folder.join(STRONGHOLD_FILENAME)
        };
        crate::stronghold::load_snapshot(&stronghold_path, stronghold_password(password)).await?;

        if self.accounts.read().await.is_empty() {
            Self::load_accounts(
                &self.accounts,
                &self.storage_path,
                self.account_options,
                self.sync_accounts_lock.clone(),
            )
            .await?;
            self.loaded_accounts = true;
            crate::spawn(Self::start_monitoring(self.accounts.clone()));
        }

        Ok(())
    }

    /// Changes the stronghold password.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    pub async fn change_stronghold_password<C: Into<String>, N: Into<String>>(
        &self,
        current_password: C,
        new_password: N,
    ) -> crate::Result<()> {
        crate::stronghold::change_password(
            &self.stronghold_snapshot_path().await?,
            stronghold_password(current_password),
            stronghold_password(new_password),
        )
        .await
        .map_err(|e| e.into())
    }

    /// Determines whether all accounts has the latest address unused.
    pub async fn is_latest_address_unused(&self) -> crate::Result<bool> {
        self.check_storage_encryption()?;
        for account_handle in self.accounts.read().await.values() {
            if !account_handle.is_latest_address_unused().await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Sets the client options for all accounts.
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        for account in self.accounts.read().await.values() {
            account.set_client_options(options.clone()).await?;
        }
        Ok(())
    }

    /// Starts the polling mechanism.
    fn start_polling(
        &mut self,
        polling_interval: Duration,
        mut stop: BroadcastReceiver<()>,
        automatic_output_consolidation: bool,
    ) {
        let storage_file_path = self.storage_path.clone();
        let accounts = self.accounts.clone();
        let account_options = self.account_options;
        let sync_accounts_lock = self.sync_accounts_lock.clone();

        let handle = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async {
                let mut interval = interval(polling_interval);
                let mut synced = false;
                let mut discovered_accounts = false;
                loop {
                    tokio::select! {
                        _ = async {
                            interval.tick().await;

                            let storage_file_path_ = storage_file_path.clone();
                            let account_options = account_options;

                            if !accounts.read().await.is_empty() {
                                match AssertUnwindSafe(
                                    poll(
                                        sync_accounts_lock.clone(),
                                        accounts.clone(),
                                        storage_file_path_,
                                        account_options,
                                        automatic_output_consolidation)
                                    )
                                    .catch_unwind()
                                    .await {
                                        Ok(response) => {
                                            if let Ok(response) = response {
                                                if response.ran_account_discovery {
                                                    discovered_accounts = true;
                                                }
                                                synced = response.synced_accounts_len > 0;
                                            }
                                        }
                                        Err(error) => {
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
        self.polling_handle.replace(handle);
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

        if let Some(mut mnemonic) = self.generated_mnemonic.take() {
            mnemonic.zeroize();
        }

        Ok(())
    }

    /// Generates a new mnemonic.
    pub fn generate_mnemonic(&mut self) -> crate::Result<String> {
        let mut entropy = [0u8; 32];
        crypto::utils::rand::fill(&mut entropy).map_err(|e| crate::Error::MnemonicEncode(format!("{:?}", e)))?;
        let mnemonic = crypto::keys::bip39::wordlist::encode(&entropy, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::MnemonicEncode(format!("{:?}", e)))?;
        self.generated_mnemonic.replace(mnemonic.clone());
        Ok(mnemonic)
    }

    /// Checks is the mnemonic is valid. If a mnemonic was generated with `generate_mnemonic()`, the mnemonic here
    /// should match the generated.
    pub fn verify_mnemonic<S: AsRef<str>>(&mut self, mnemonic: S) -> crate::Result<()> {
        // first we check if the mnemonic is valid to give meaningful errors
        crypto::keys::bip39::wordlist::verify(mnemonic.as_ref(), &crypto::keys::bip39::wordlist::ENGLISH)
            // TODO: crypto::bip39::wordlist::Error should impl Display
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;

        // then we check if the provided mnemonic matches the mnemonic generated with `generate_mnemonic`
        if let Some(generated_mnemonic) = &self.generated_mnemonic {
            if generated_mnemonic != mnemonic.as_ref() {
                return Err(crate::Error::InvalidMnemonic(
                    "doesn't match the generated mnemonic".to_string(),
                ));
            }
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
            self.account_options,
            self.sync_accounts_lock.clone(),
        ))
    }

    /// Deletes an account.
    pub async fn remove_account<I: Into<AccountIdentifier>>(&self, account_id: I) -> crate::Result<()> {
        self.check_storage_encryption()?;

        let account_id = {
            let account_handle = self.get_account(account_id).await?;
            let account = account_handle.read().await;

            if account.balance().await?.total > 0 {
                return Err(crate::Error::AccountNotEmpty);
            }

            account.id().to_string()
        };

        self.accounts.write().await.remove(&account_id);

        crate::storage::get(&self.storage_path)
            .await?
            .lock()
            .await
            .remove_account(&account_id)
            .await?;

        Ok(())
    }

    /// Syncs all accounts.
    pub fn sync_accounts(&self) -> crate::Result<AccountsSynchronizer> {
        self.check_storage_encryption()?;
        Ok(AccountsSynchronizer::new(
            self.sync_accounts_lock.clone(),
            self.accounts.clone(),
            self.storage_path.clone(),
            self.account_options,
        ))
    }

    /// Transfers an amount from an account to another.
    pub async fn internal_transfer<F: Into<AccountIdentifier>, T: Into<AccountIdentifier>>(
        &self,
        from_account_id: F,
        to_account_id: T,
        amount: NonZeroU64,
    ) -> crate::Result<Message> {
        self.check_storage_encryption()?;

        let to_account_handle = self.get_account(to_account_id).await?;
        let to_address = to_account_handle.read().await.latest_address().address().clone();

        let message = self
            .get_account(from_account_id)
            .await?
            .transfer(Transfer::builder(to_address, amount).finish())
            .await?;

        // store the message on the receive account
        let mut message_ = message.clone();
        if let Some(MessagePayload::Transaction(tx)) = message_.payload.as_mut() {
            let TransactionEssence::Regular(essence) = tx.essence_mut();
            essence.incoming = true;
        }
        to_account_handle.write().await.save_messages(vec![message_]).await?;

        Ok(message)
    }

    /// Backups the storage to the given destination
    pub async fn backup<P: AsRef<Path>>(&self, destination: P, stronghold_password: String) -> crate::Result<PathBuf> {
        let destination = destination.as_ref().to_path_buf();
        if !(destination.is_dir() || destination.parent().map(|parent| parent.is_dir()).unwrap_or_default()) {
            return Err(crate::Error::InvalidBackupDestination);
        }

        let storage_path = {
            // create a account manager to setup the stronghold storage for the backup
            let mut manager = Self::builder()
                .with_storage(&self.storage_folder, None)
                .unwrap() // safe to unwrap - password is None
                .skip_polling()
                .with_stronghold_storage()
                .finish()
                .await?;
            manager.accounts = self.accounts.clone(); // force manager to skip loading accounts
            manager.set_stronghold_password(stronghold_password).await?;
            let stronghold_storage_path = self.storage_folder.join(STRONGHOLD_FILENAME);
            let stronghold_storage = crate::storage::get(&stronghold_storage_path).await?;

            for (account_id, account_handle) in self.accounts.read().await.iter() {
                let mut account = account_handle.write().await;
                stronghold_storage
                    .lock()
                    .await
                    .save_account(account_id, &account)
                    .await?;
                let messages = account.list_messages(0, 0, None).await?;
                // switch account storage_path to stronghold to save the messages
                account.set_storage_path(stronghold_storage_path.clone());
                account.save_messages(messages).await?;
                // revert to original storage_path
                account.set_storage_path(self.storage_path.clone());
            }
            self.storage_folder.join(STRONGHOLD_FILENAME)
        };

        let destination = if let Some(filename) = storage_path.file_name() {
            let destination = if destination.is_dir() {
                destination.join(backup_filename(filename.to_str().unwrap()))
            } else {
                destination
            };
            let res = fs::copy(storage_path, &destination);

            let mut stronghold_storage = crate::storage::stronghold::StrongholdStorageAdapter::new(
                &self.storage_folder.join(STRONGHOLD_FILENAME),
            )
            // stronghold adapter `new` never fails
            .unwrap();
            for account_handle in self.accounts.read().await.values() {
                stronghold_storage.remove(&account_handle.read().await.id()).await?;
            }

            res?;
            destination
        } else {
            return Err(crate::Error::StorageDoesntExist);
        };
        Ok(destination)
    }

    /// Import backed up accounts.
    pub async fn import_accounts<S: AsRef<Path>>(
        &mut self,
        source: S,
        stronghold_password: String,
    ) -> crate::Result<()> {
        let source = source.as_ref();
        if source.is_dir() || !source.exists() || source.extension().unwrap_or_default() != "stronghold" {
            return Err(crate::Error::InvalidBackupFile);
        }
        if !self.accounts.read().await.is_empty() {
            return Err(crate::Error::StorageExists);
        }

        let storage_file_path = self.storage_folder.join(ROCKSDB_FILENAME);

        fs::create_dir_all(&self.storage_folder)?;

        let mut stronghold_manager = Self::builder()
            .with_storage(source.parent().unwrap(), None)
            .unwrap() // safe to unwrap - password is None
            .with_storage_file_name(
                source
                    .file_name()
                    .unwrap()
                    .to_str()
                    .ok_or(crate::Error::InvalidBackupFile)?,
            ) // safe to unwrap since we checked the path
            .skip_polling()
            .with_stronghold_storage()
            .finish()
            .await?;
        stronghold_manager
            .set_stronghold_password(stronghold_password.clone())
            .await?;
        let mut import_data = Vec::new();
        for (id, account) in stronghold_manager.accounts.read().await.iter() {
            self.accounts.write().await.insert(id.clone(), account.clone());
            import_data.push((account.clone(), account.list_messages(0, 0, None).await?));
        }
        self.set_stronghold_password(stronghold_password.clone()).await?;
        for (account_handle, messages) in import_data {
            let mut account = account_handle.write().await;
            account.set_storage_path(self.storage_path.clone());
            account.save().await?;
            account.save_messages(messages).await?;
            account.cached_messages = Default::default();
        }
        // wait for stronghold to finish its tasks
        let _ = crate::stronghold::actor_runtime().lock().await;
        fs::copy(source, self.storage_folder.join(STRONGHOLD_FILENAME))?;

        #[cfg(feature = "stronghold")]
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
                        associated_account.replace(account_handle);
                    }
                }
                associated_account
            }
            AccountIdentifier::Alias(alias) => {
                let mut associated_account = None;
                for account_handle in accounts.values() {
                    let account = account_handle.read().await;
                    if account.alias() == &alias {
                        associated_account.replace(account_handle);
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
                        associated_account.replace(account_handle);
                        break;
                    }
                }
                associated_account
            }
        };

        account.cloned().ok_or(crate::Error::RecordNotFound)
    }

    /// Gets all accounts from storage.
    pub async fn get_accounts(&self) -> crate::Result<Vec<AccountHandle>> {
        self.check_storage_encryption()?;
        let mut accounts = Vec::new();
        for account in self.accounts.read().await.values() {
            let index = account.index().await;
            accounts.push((index, account.clone()));
        }
        accounts.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(accounts.into_iter().map(|(_, account)| account).collect())
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        self.get_account(account_id).await?.reattach(message_id).await
    }

    /// Promotes an unconfirmed transaction.
    pub async fn promote<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        self.get_account(account_id).await?.promote(message_id).await
    }

    /// Retries an unconfirmed transaction.
    pub async fn retry<I: Into<AccountIdentifier>>(
        &self,
        account_id: I,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        self.get_account(account_id).await?.retry(message_id).await
    }

    /// Get seed checksum
    pub fn get_seed_checksum(seed: String) -> crate::Result<String> {
        iota_migration::client::migration::get_seed_checksum(seed)
            .map_err(|e| crate::Error::LegacyClientError(Box::new(e)))
    }
}

macro_rules! event_getters_impl {
    ($event_ty:ty, $get_fn_name: ident, $get_count_fn_name: ident) => {
        impl AccountManager {
            /// Gets the paginated events with an optional timestamp filter.
            pub async fn $get_fn_name<T: Into<Option<Timestamp>>>(
                &self,
                count: usize,
                skip: usize,
                from_timestamp: T,
            ) -> crate::Result<Vec<$event_ty>> {
                crate::storage::get(&self.storage_path)
                    .await?
                    .lock()
                    .await
                    .$get_fn_name(count, skip, from_timestamp)
                    .await
            }

            /// Gets the count of events with an optional timestamp filter.
            pub async fn $get_count_fn_name<T: Into<Option<Timestamp>>>(
                &self,
                from_timestamp: T,
            ) -> crate::Result<usize> {
                let count = crate::storage::get(&self.storage_path)
                    .await?
                    .lock()
                    .await
                    .$get_count_fn_name(from_timestamp)
                    .await?;
                Ok(count)
            }
        }
    };
}

event_getters_impl!(BalanceEvent, get_balance_change_events, get_balance_change_event_count);
event_getters_impl!(
    TransactionConfirmationChangeEvent,
    get_transaction_confirmation_events,
    get_transaction_confirmation_event_count
);
event_getters_impl!(
    TransactionEvent,
    get_new_transaction_events,
    get_new_transaction_event_count
);
event_getters_impl!(
    TransactionReattachmentEvent,
    get_reattachment_events,
    get_reattachment_event_count
);
event_getters_impl!(TransactionEvent, get_broadcast_events, get_broadcast_event_count);

/// The accounts synchronizer.
pub struct AccountsSynchronizer {
    mutex: Arc<Mutex<()>>,
    accounts: AccountStore,
    storage_file_path: PathBuf,
    address_index: Option<usize>,
    gap_limit: Option<usize>,
    account_options: AccountOptions,
    discover_accounts: bool,
    account_discovery_threshold: usize,
    skip_change_addresses: bool,
    ran_account_discovery: bool,
    steps: Option<Vec<AccountSynchronizeStep>>,
}

impl AccountsSynchronizer {
    pub(crate) fn new(
        mutex: Arc<Mutex<()>>,
        accounts: AccountStore,
        storage_file_path: PathBuf,
        account_options: AccountOptions,
    ) -> Self {
        Self {
            mutex,
            accounts,
            storage_file_path,
            address_index: None,
            gap_limit: None,
            account_options,
            discover_accounts: true,
            account_discovery_threshold: 1,
            skip_change_addresses: false,
            ran_account_discovery: false,
            steps: None,
        }
    }

    /// Number of address indexes that are generated.
    pub fn gap_limit(mut self, limit: usize) -> Self {
        self.gap_limit.replace(limit);
        self
    }

    /// Initial address index to start syncing.
    pub fn address_index(mut self, address_index: usize) -> Self {
        self.address_index.replace(address_index);
        self
    }

    /// Skips the account discovery process.
    pub fn skip_account_discovery(mut self) -> Self {
        self.discover_accounts = false;
        self
    }

    /// Skip syncing existing change addresses.
    pub fn skip_change_addresses(mut self) -> Self {
        self.skip_change_addresses = true;
        self
    }

    /// Sets the minimum number of accounts to check on the discovery process.
    pub fn account_discovery_threshold(mut self, account_discovery_threshold: usize) -> Self {
        self.account_discovery_threshold = account_discovery_threshold;
        self
    }

    /// Sets the steps to run on the sync process.
    /// By default it runs all steps (sync_addresses and sync_messages),
    /// but the library can pick what to run here.
    pub(crate) fn steps(mut self, steps: Vec<AccountSynchronizeStep>) -> Self {
        self.steps.replace(steps);
        self
    }

    /// Syncs the accounts with the Tangle.
    pub async fn execute(&mut self) -> crate::Result<Vec<SyncedAccount>> {
        let accounts = self.accounts.clone();
        for account_handle in accounts.read().await.values() {
            account_handle.disable_mqtt();
        }
        let result = self.execute_internal().await;
        for account_handle in accounts.read().await.values() {
            account_handle.enable_mqtt();
        }
        result
    }

    async fn execute_internal(&mut self) -> crate::Result<Vec<SyncedAccount>> {
        let _lock = self.mutex.lock().await;

        let mut tasks = Vec::new();
        {
            let accounts = self.accounts.read().await;
            let address_index = self.address_index;
            let gap_limit = self.gap_limit;
            let skip_change_addresses = self.skip_change_addresses;
            for account_handle in accounts.values() {
                let account_handle = account_handle.clone();
                let steps = self.steps.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let mut sync = account_handle.sync().await;
                        if skip_change_addresses {
                            sync = sync.skip_change_addresses();
                        }
                        if let Some(index) = address_index {
                            sync = sync.address_index(index);
                        }
                        if let Some(limit) = gap_limit {
                            sync = sync.gap_limit(limit);
                        }
                        if let Some(steps) = steps {
                            sync = sync.steps(steps);
                        }
                        let synced_data = sync.get_new_history().await?;
                        crate::Result::Ok((account_handle, synced_data))
                    })
                    .await
                });
            }
        }

        let mut synced_data = Vec::new();
        for res in futures::future::try_join_all(tasks)
            .await
            .expect("failed to sync accounts")
        {
            let (account_handle, data) = res?;
            let account_handle_ = account_handle.clone();
            let mut account = account_handle_.write().await;
            let addresses_before_sync: Vec<(String, u64, HashMap<OutputId, AddressOutput>)> = account
                .addresses()
                .iter()
                .map(|a| (a.address().to_bech32(), a.balance(), a.outputs().clone()))
                .collect();
            account.append_addresses(data.addresses.to_vec());
            synced_data.push((account_handle, addresses_before_sync, data));
        }

        let mut synced_accounts = Vec::new();
        let mut last_account = None;
        let mut last_account_index = 0;
        for (account_handle, _, _) in &synced_data {
            let account = account_handle.read().await;
            if *account.index() >= last_account_index {
                last_account_index = *account.index();
                last_account.replace((
                    account
                        .addresses()
                        .iter()
                        .all(|addr| addr.balance() == 0 && addr.outputs().is_empty()),
                    account.client_options().clone(),
                    account.signer_type().clone(),
                ));
            }
        }

        let discovered_accounts_res = match last_account {
            Some((is_empty, client_options, signer_type)) => {
                if !is_empty || self.account_discovery_threshold > 0 {
                    if self.discover_accounts {
                        log::debug!("[SYNC] running account discovery because the latest account is not empty");
                        discover_accounts(
                            self.accounts.clone(),
                            self.account_discovery_threshold,
                            self.gap_limit,
                            &self.storage_file_path,
                            &client_options,
                            Some(signer_type),
                            self.account_options,
                            self.mutex.clone(),
                        )
                        .await
                    } else {
                        Ok(vec![])
                    }
                } else {
                    log::debug!("[SYNC] skipping account discovery because the latest account is empty");
                    Ok(vec![])
                }
            }
            None => Ok(vec![]),
        };

        let mut discovered_account_ids = Vec::new();
        self.ran_account_discovery = discovered_accounts_res.is_ok();
        if let Ok(discovered_accounts) = discovered_accounts_res {
            if !discovered_accounts.is_empty() {
                let mut accounts = self.accounts.write().await;
                for (account_handle, synced_account_data) in discovered_accounts {
                    let account_handle_ = account_handle.clone();
                    let mut account = account_handle_.write().await;
                    account.set_skip_persistence(false);
                    account.set_addresses(synced_account_data.addresses.to_vec());
                    account.save().await?;
                    accounts.insert(account.id().clone(), account_handle.clone());
                    discovered_account_ids.push(account.id().clone());
                    synced_data.push((account_handle, Vec::new(), synced_account_data));
                }
            }
        }

        for (account_handle, addresses_before_sync, data) in synced_data {
            let (parsed_messages, messages_before_sync) = {
                let mut account = account_handle.write().await;
                let messages_before_sync: Vec<(MessageId, Option<bool>)> = account
                    .with_messages(|messages| messages.iter().map(|m| (m.key, m.confirmed)).collect())
                    .await;

                let parsed_messages = data.parse_messages(account_handle.accounts.clone(), &account).await?;
                account.save_messages(parsed_messages.to_vec()).await?;
                account.set_last_synced_at(Some(chrono::Local::now()));
                account.save().await?;
                (parsed_messages, messages_before_sync)
            };

            let mut new_messages = Vec::new();
            let mut confirmation_changed_messages = Vec::new();
            for message in parsed_messages {
                if !messages_before_sync.iter().any(|(id, _)| id == message.id()) {
                    new_messages.push(message.clone());
                }
                if messages_before_sync
                    .iter()
                    .any(|(id, confirmed)| id == message.id() && confirmed != message.confirmed())
                {
                    confirmation_changed_messages.push(message);
                }
            }

            let account = account_handle.read().await;

            if !discovered_account_ids.contains(account.id()) {
                let persist_events = account_handle.account_options.persist_events;
                let events = AccountSynchronizer::get_events(
                    account_handle.account_options,
                    &addresses_before_sync,
                    account.addresses(),
                    &new_messages,
                    &confirmation_changed_messages,
                )
                .await?;
                for message in events.new_transaction_events {
                    emit_transaction_event(TransactionEventType::NewTransaction, &account, message, persist_events)
                        .await?;
                }
                for confirmation_change_event in events.confirmation_change_events {
                    emit_confirmation_state_change(
                        &account,
                        confirmation_change_event.message,
                        confirmation_change_event.confirmed,
                        persist_events,
                    )
                    .await?;
                }
                for balance_change_event in events.balance_change_events {
                    emit_balance_change(
                        &account,
                        &balance_change_event.address,
                        balance_change_event.message_id,
                        balance_change_event.balance_change,
                        persist_events,
                    )
                    .await?;
                }
            }
            drop(account);

            let mut synced_account = SyncedAccount::from(account_handle.clone()).await;
            let mut updated_messages = new_messages;
            updated_messages.extend(confirmation_changed_messages);
            synced_account.messages = updated_messages;

            let account = account_handle.read().await;
            synced_account.addresses = account
                .addresses()
                .iter()
                .filter(|a| {
                    match addresses_before_sync
                        .iter()
                        .find(|(addr, _, _)| addr == &a.address().to_bech32())
                    {
                        Some((_, balance, outputs)) => *balance != a.balance() || outputs != a.outputs(),
                        None => true,
                    }
                })
                .cloned()
                .collect();
            synced_accounts.push(synced_account);
        }

        Ok(synced_accounts)
    }
}

struct PollResponse {
    ran_account_discovery: bool,
    synced_accounts_len: usize,
}

async fn poll(
    sync_accounts_lock: Arc<Mutex<()>>,
    accounts: AccountStore,
    storage_file_path: PathBuf,
    account_options: AccountOptions,
    automatic_output_consolidation: bool,
) -> crate::Result<PollResponse> {
    let mut synchronizer =
        AccountsSynchronizer::new(sync_accounts_lock, accounts.clone(), storage_file_path, account_options);
    synchronizer = synchronizer.skip_account_discovery().skip_change_addresses();
    let synced_accounts = synchronizer.execute().await?;

    log::debug!("[POLLING] synced accounts");

    let retried = retry_unconfirmed_transactions(&synced_accounts).await?;
    consolidate_outputs_if_needed(automatic_output_consolidation, &synced_accounts).await?;

    for retried_data in retried {
        let mut account = retried_data.account_handle.write().await;
        let client = crate::client::get_client(account.client_options()).await?;

        for (reattached_message_id, message) in &retried_data.reattached {
            emit_reattachment_event(
                &account,
                *reattached_message_id,
                &message,
                retried_data.account_handle.account_options.persist_events,
            )
            .await?;
            let mut reattached_message = account.get_message(reattached_message_id).await.unwrap();
            reattached_message.set_reattachment_message_id(Some(*message.id()));
            account.save_messages(vec![reattached_message]).await?;
        }

        let mut messages_to_save: Vec<Message> = retried_data
            .reattached
            .into_iter()
            .map(|(_, message)| message)
            .collect();
        messages_to_save.extend(retried_data.promoted);
        account.save_messages(messages_to_save).await?;

        for message_id in retried_data.no_need_promote_or_reattach {
            let mut message = account.get_message(&message_id).await.unwrap();
            if let Ok(metadata) = client.read().await.get_message().metadata(&message_id).await {
                if let Some(ledger_inclusion_state) = metadata.ledger_inclusion_state {
                    let confirmed = ledger_inclusion_state == LedgerInclusionStateDto::Included;
                    if message.confirmed() != &Some(confirmed) {
                        message.set_confirmed(Some(confirmed));
                        account.save_messages(vec![message.clone()]).await?;
                        emit_confirmation_state_change(
                            &account,
                            message,
                            confirmed,
                            retried_data.account_handle.account_options.persist_events,
                        )
                        .await?;
                    }
                }
            }
        }
        account.save().await?;
    }
    Ok(PollResponse {
        ran_account_discovery: synchronizer.ran_account_discovery,
        synced_accounts_len: synced_accounts.len(),
    })
}

#[allow(clippy::too_many_arguments)]
async fn discover_accounts(
    accounts: AccountStore,
    threshold: usize,
    gap_limit: Option<usize>,
    storage_path: &Path,
    client_options: &ClientOptions,
    signer_type: Option<SignerType>,
    account_options: AccountOptions,
    sync_accounts_lock: Arc<Mutex<()>>,
) -> crate::Result<Vec<(AccountHandle, SyncedAccountData)>> {
    let mut synced_accounts = vec![];
    let last_account_index = accounts.read().await.len() - 1;
    let mut index = last_account_index + 1;
    loop {
        let mut account_initialiser = AccountInitialiser::new(
            client_options.clone(),
            accounts.clone(),
            storage_path.to_path_buf(),
            account_options,
            sync_accounts_lock.clone(),
        )
        .skip_persistence()
        .index(index);
        if let Some(signer_type) = &signer_type {
            account_initialiser = account_initialiser.signer_type(signer_type.clone());
        }
        let account_handle = account_initialiser.initialise().await?;
        log::debug!(
            "[SYNC] discovering account {}, signer type {:?}",
            account_handle.read().await.alias(),
            account_handle.read().await.signer_type()
        );
        let mut synchronizer = account_handle.sync().await;
        if let Some(gap_limit) = gap_limit {
            synchronizer = synchronizer.gap_limit(gap_limit);
        }
        match synchronizer.get_new_history().await {
            Ok(synced_account_data) => {
                let is_empty = synced_account_data
                    .addresses
                    .iter()
                    .all(|a| a.balance() == 0 && a.outputs().is_empty());
                log::debug!("[SYNC] discovered account is empty? {}", is_empty);
                if is_empty {
                    if index - last_account_index >= threshold {
                        break;
                    }
                } else {
                    synced_accounts.push((account_handle, synced_account_data));
                }
                index += 1;
            }
            Err(e) => {
                log::error!("[SYNC] failed to sync to discover account: {:?}", e);
                // break if the account failed to sync
                // this ensures that the previously discovered accounts get stored.
                break;
            }
        }
    }
    Ok(synced_accounts)
}

struct RetriedData {
    promoted: Vec<Message>,
    reattached: Vec<(MessageId, Message)>,
    no_need_promote_or_reattach: Vec<MessageId>,
    account_handle: AccountHandle,
}

#[allow(unused_mut)]
async fn consolidate_outputs_if_needed(
    mut automatic_consolidation: bool,
    synced_accounts: &[SyncedAccount],
) -> crate::Result<()> {
    for synced in synced_accounts {
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        {
            let account = synced.account_handle.read().await;
            let signer_type = account.signer_type();
            if signer_type == &SignerType::LedgerNano || signer_type == &SignerType::LedgerNanoSimulator {
                let addresses = synced.account_handle.output_consolidation_addresses().await?;
                for address in addresses {
                    crate::event::emit_address_consolidation_needed(&account, address).await;
                }
                // on ledger we do not consolidate outputs automatically
                automatic_consolidation = false;
            }
        }
        if automatic_consolidation {
            synced.consolidate_outputs().await?;
        }
    }
    Ok(())
}

async fn retry_unconfirmed_transactions(synced_accounts: &[SyncedAccount]) -> crate::Result<Vec<RetriedData>> {
    let mut retried_messages = vec![];
    for synced in synced_accounts {
        let unconfirmed_messages: Vec<(MessageId, Option<MessagePayload>)> = synced
            .account_handle()
            .read()
            .await
            .list_messages(0, 0, Some(MessageType::Unconfirmed))
            .await?
            .iter()
            .map(|message| (*message.id(), message.payload().clone()))
            .collect();
        let mut reattachments = Vec::new();
        let mut promotions = Vec::new();
        let mut no_need_promote_or_reattach = Vec::new();
        for (message_id, message_payload) in unconfirmed_messages {
            log::debug!("[POLLING] retrying {:?}", message_id);
            match synced.retry(&message_id).await {
                Ok(new_message) => {
                    // if the payload is the same, it was reattached; otherwise it was promoted
                    if new_message.payload() == &message_payload {
                        log::debug!("[POLLING] rettached and new message is {:?}", new_message);
                        reattachments.push((message_id, new_message));
                    } else {
                        log::debug!("[POLLING] promoted and new message is {:?}", new_message);
                        promotions.push(new_message);
                    }
                }
                Err(crate::Error::ClientError(ref e)) => {
                    if let iota::client::Error::NoNeedPromoteOrReattach(_) = e.as_ref() {
                        no_need_promote_or_reattach.push(message_id);
                    }
                }
                _ => {}
            }
        }
        retried_messages.push(RetriedData {
            promoted: promotions,
            reattached: reattachments,
            no_need_promote_or_reattach,
            account_handle: synced.account_handle().clone(),
        });
    }
    Ok(retried_messages)
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
        address::{AddressBuilder, AddressOutput, AddressWrapper, IotaAddress, OutputKind},
        client::ClientOptionsBuilder,
        event::*,
        message::Message,
    };
    use iota::{Ed25519Address, IndexationPayload, MessageBuilder, MessageId, Parents, Payload, TransactionId};
    use std::{collections::HashMap, path::PathBuf};

    #[tokio::test]
    async fn store_accounts() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

            manager
                .remove_account(account_handle.read().await.id())
                .await
                .expect("failed to remove account");
        })
        .await;
    }

    #[tokio::test]
    async fn delete_storage() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            crate::test_utils::AccountCreator::new(&manager).create().await;
            manager
                .delete()
                .await
                .map_err(|(e, _)| e)
                .expect("failed to delete storage");
        })
        .await;
    }

    #[tokio::test]
    async fn duplicated_alias() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.testnet.chrysalis2.com")
            .expect("invalid node URL")
            .build()
            .unwrap();
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
    #[ignore]
    async fn get_account() {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.testnet.chrysalis2.com")
            .expect("invalid node URL")
            .build()
            .unwrap();

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
            let mut outputs = HashMap::default();
            let output = AddressOutput {
                transaction_id: TransactionId::new([0; 32]),
                message_id: MessageId::new([0; 32]),
                index: 0,
                amount: 5,
                is_spent: false,
                address: crate::test_utils::generate_random_iota_address(),
                kind: OutputKind::SignatureLockedSingle,
            };
            outputs.insert(output.id().unwrap(), output);
            for address in account.addresses_mut() {
                address.set_outputs(outputs.clone());
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
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
                .expect("invalid node URL")
                .build()
                .unwrap();

            let account_handle = manager
                .create_account(client_options)
                .unwrap()
                .messages(vec![Message::from_iota_message(
                    MessageId::new([0; 32]),
                    MessageBuilder::new()
                        .with_nonce_provider(crate::test_utils::NoopNonceProvider {}, 4000f64)
                        .with_parents(Parents::new(vec![MessageId::new([0; 32])]).unwrap())
                        .with_payload(Payload::Indexation(Box::new(
                            IndexationPayload::new(b"index", &[0; 16]).unwrap(),
                        )))
                        .with_network_id(0)
                        .finish()
                        .unwrap(),
                    super::AccountStore::new(Default::default()),
                    "",
                    &[crate::test_utils::generate_random_address()],
                    &ClientOptionsBuilder::new().build().unwrap(),
                )
                .with_confirmed(Some(true))
                .finish()
                .await
                .unwrap()])
                .initialise()
                .await
                .unwrap();

            let remove_response = manager.remove_account(account_handle.read().await.id()).await;
            assert!(remove_response.is_ok());
        })
        .await;
    }

    #[tokio::test]
    async fn remove_account_with_balance() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
                .expect("invalid node URL")
                .build()
                .unwrap();

            let account_handle = manager
                .create_account(client_options)
                .unwrap()
                .addresses(vec![AddressBuilder::new()
                    .key_index(0)
                    .address(AddressWrapper::new(
                        IotaAddress::Ed25519(Ed25519Address::new([0; 32])),
                        "atoi".to_string(),
                    ))
                    .outputs(vec![AddressOutput {
                        transaction_id: TransactionId::new([0; 32]),
                        message_id: MessageId::new([0; 32]),
                        index: 0,
                        amount: 5,
                        is_spent: false,
                        address: crate::test_utils::generate_random_iota_address(),
                        kind: OutputKind::SignatureLockedSingle,
                    }])
                    .build()
                    .unwrap()])
                .initialise()
                .await
                .unwrap();
            let account = account_handle.read().await;

            let remove_response = manager.remove_account(account.id()).await;
            assert!(remove_response.is_err());
        })
        .await;
    }

    #[tokio::test]
    async fn create_account_with_latest_without_history() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
                .expect("invalid node URL")
                .build()
                .unwrap();

            manager
                .create_account(client_options.clone())
                .unwrap()
                .alias("alias")
                .initialise()
                .await
                .expect("failed to add account");

            let create_response = manager.create_account(client_options).unwrap().initialise().await;
            assert!(create_response.is_err());
        })
        .await;
    }

    #[tokio::test]
    async fn create_account_skip_persistence() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
                .expect("invalid node URL")
                .with_network("testnet")
                .build()
                .unwrap();

            let account_handle = manager
                .create_account(client_options.clone())
                .unwrap()
                .skip_persistence()
                .initialise()
                .await
                .expect("failed to add account");

            let account_get_res = manager.get_account(account_handle.read().await.id()).await;
            assert!(account_get_res.is_err(), "{}", true);
            match account_get_res.unwrap_err() {
                crate::Error::RecordNotFound => {}
                _ => panic!("unexpected get_account response; expected RecordNotFound"),
            }
        })
        .await;
    }

    #[tokio::test]
    async fn backup_and_restore_happy_path() {
        let backup_path = "./backup/happy-path";
        let _ = std::fs::remove_dir_all(backup_path);
        std::fs::create_dir_all(backup_path).unwrap();

        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

            // backup the stored accounts to ./backup/happy-path/${backup_name}
            let backup_path = manager.backup(backup_path, "password".to_string()).await.unwrap();
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
            assert_eq!(backup_file_path.extension().unwrap_or_default(), "stronghold");

            let is_encrypted = crate::storage::get(manager.storage_path())
                .await
                .unwrap()
                .lock()
                .await
                .is_encrypted();

            // get another manager instance so we can import the accounts to a different storage
            #[allow(unused_mut)]
            let mut manager = crate::test_utils::get_account_manager().await;

            #[cfg(feature = "stronghold")]
            {
                // wait for stronghold to finish pending operations and delete the storage file
                crate::stronghold::unload_snapshot(&manager.stronghold_snapshot_path().await.unwrap(), false)
                    .await
                    .unwrap();
                let _ = crate::stronghold::actor_runtime().lock().await;

                if crate::storage::get(manager.storage_path())
                    .await
                    .unwrap()
                    .lock()
                    .await
                    .id()
                    == crate::storage::stronghold::STORAGE_ID
                {
                    let _ = std::fs::remove_file(manager.storage_path());
                }
            }

            if is_encrypted {
                manager.set_storage_password("password").await.unwrap();
            }

            // import the accounts from the backup and assert that it's the same
            manager
                .import_accounts(&backup_file_path, "password".to_string())
                .await
                .unwrap();
            assert!(manager.stronghold_snapshot_path().await.unwrap().exists(), "{}", true);

            let imported_account = manager.get_account(account_handle.read().await.id()).await.unwrap();
            // set the account storage path field so the assert works
            account_handle
                .write()
                .await
                .set_storage_path(manager.storage_path().clone());
            assert_eq!(&*account_handle.read().await, &*imported_account.read().await);
        })
        .await;
    }

    #[tokio::test]
    async fn backup_and_restore_storage_already_exists() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |mut manager, _| async move {
            let backup_path = PathBuf::from("./backup/account-exists");
            let _ = std::fs::remove_dir_all(&backup_path);
            std::fs::create_dir_all(&backup_path).unwrap();
            // first we'll create an example account
            let address = crate::test_utils::generate_random_iota_address();
            let address = AddressBuilder::new()
                .address(address.clone())
                .key_index(0)
                .outputs(vec![])
                .build()
                .unwrap();
            crate::test_utils::AccountCreator::new(&manager)
                .addresses(vec![address])
                .create()
                .await;

            let backup_file_path = backup_path.join("wallet.stronghold");
            let backup_path = manager.backup(&backup_file_path, "password".to_string()).await.unwrap();
            assert_eq!(backup_path, backup_file_path);

            let response = manager.import_accounts(&backup_file_path, "password".to_string()).await;

            assert!(response.is_err());
            assert!(matches!(response.unwrap_err(), crate::Error::StorageExists));
        })
        .await;
    }

    #[tokio::test]
    async fn storage_password_reencrypt() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |mut manager, _| async move {
            crate::test_utils::AccountCreator::new(&manager).create().await;
            manager.set_storage_password("new-password").await.unwrap();
            let account_store = super::AccountStore::new(Default::default());
            super::AccountManager::load_accounts(
                &account_store,
                manager.storage_path(),
                manager.account_options,
                Default::default(),
            )
            .await
            .unwrap();
            assert_eq!(account_store.read().await.len(), 1);
        })
        .await;
    }

    #[tokio::test]
    async fn get_balance_change_events() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
            let account = account_handle.read().await;
            let change_events = vec![
                BalanceChange::spent(0),
                BalanceChange::spent(1),
                BalanceChange::spent(2),
                BalanceChange::spent(3),
            ];
            for change in &change_events {
                emit_balance_change(&account, account.latest_address().address(), None, *change, true)
                    .await
                    .unwrap();
            }
            assert!(
                manager.get_balance_change_event_count(None).await.unwrap() == change_events.len(),
                "{}",
                true
            );
            for (take, skip) in &[(2, 0), (2, 2)] {
                let found = manager
                    .get_balance_change_events(*take, *skip, None)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| e.balance_change)
                    .collect::<Vec<BalanceChange>>();
                let expected = change_events
                    .clone()
                    .into_iter()
                    .skip(*skip)
                    .take(*take)
                    .collect::<Vec<BalanceChange>>();
                assert!(found == expected, "{}", true);
            }
        })
        .await;
    }

    #[tokio::test]
    async fn get_transaction_confirmation_events() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
            let account = account_handle.read().await;
            let m1 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let m2 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let m3 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let confirmation_change_events = vec![
                (m1, true),
                (
                    crate::test_utils::GenerateMessageBuilder::default().build().await,
                    false,
                ),
                (m2, false),
                (m3, true),
            ];
            for (message, change) in &confirmation_change_events {
                emit_confirmation_state_change(&account, message.clone(), *change, true)
                    .await
                    .unwrap();
            }
            assert!(
                manager.get_transaction_confirmation_event_count(None).await.unwrap()
                    == confirmation_change_events.len(),
                "{}",
                true
            );
            for (take, skip) in &[(2, 0), (2, 2)] {
                let found = manager
                    .get_transaction_confirmation_events(*take, *skip, None)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| (e.message, e.confirmed))
                    .collect::<Vec<(Message, bool)>>();
                let expected = confirmation_change_events
                    .clone()
                    .into_iter()
                    .skip(*skip)
                    .take(*take)
                    .collect::<Vec<(Message, bool)>>();
                assert!(found == expected, "{}", true);
            }
        })
        .await;
    }

    #[tokio::test]
    async fn get_reattachment_events() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
            let account = account_handle.read().await;
            let m1 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let m2 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let m3 = crate::test_utils::GenerateMessageBuilder::default().build().await;
            let reattachment_events = vec![
                m1,
                crate::test_utils::GenerateMessageBuilder::default().build().await,
                m2,
                m3,
            ];
            for message in &reattachment_events {
                emit_reattachment_event(&account, *message.id(), message, true)
                    .await
                    .unwrap();
            }
            assert!(
                manager.get_reattachment_event_count(None).await.unwrap() == reattachment_events.len(),
                "{}",
                true
            );
            for (take, skip) in &[(2, 0), (2, 2)] {
                let found = manager
                    .get_reattachment_events(*take, *skip, None)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<Message>>();
                let expected = reattachment_events
                    .clone()
                    .into_iter()
                    .skip(*skip)
                    .take(*take)
                    .collect::<Vec<Message>>();
                assert!(found == expected, "{}", true);
            }
        })
        .await;
    }

    macro_rules! transaction_event_test {
        ($event_type: expr, $count_get_fn: ident, $get_fn: ident) => {
            #[tokio::test]
            async fn $get_fn() {
                crate::test_utils::with_account_manager(
                    crate::test_utils::TestType::Storage,
                    |manager, _| async move {
                        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                        let account = account_handle.read().await;
                        let m1 = crate::test_utils::GenerateMessageBuilder::default().build().await;
                        let m2 = crate::test_utils::GenerateMessageBuilder::default().build().await;
                        let m3 = crate::test_utils::GenerateMessageBuilder::default().build().await;
                        let m4 = crate::test_utils::GenerateMessageBuilder::default().build().await;
                        let events = vec![m1, m2, m3, m4];
                        for message in &events {
                            emit_transaction_event($event_type, &account, message.clone(), true)
                                .await
                                .unwrap();
                        }
                        assert!(
                            manager.$count_get_fn(None).await.unwrap() == events.len(),
                            "{}",
                            true
                        );
                        for (take, skip) in &[(2, 0), (2, 2)] {
                            let found = manager
                                .$get_fn(*take, *skip, None)
                                .await
                                .unwrap()
                                .into_iter()
                                .map(|e| e.message)
                                .collect::<Vec<Message>>();
                            let expected = events
                                .clone()
                                .into_iter()
                                .skip(*skip)
                                .take(*take)
                                .collect::<Vec<Message>>();
                            assert!(found == expected, "{}", true);
                        }
                    },
                )
                .await;
            }
        };
    }

    transaction_event_test!(
        TransactionEventType::NewTransaction,
        get_new_transaction_event_count,
        get_new_transaction_events
    );
    transaction_event_test!(
        TransactionEventType::Broadcast,
        get_broadcast_event_count,
        get_broadcast_events
    );
}

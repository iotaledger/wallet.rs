// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_manager::{AccountOptions, AccountStore},
    address::{Address, AddressBuilder, AddressOutput, AddressWrapper},
    client::{ClientOptions, Node},
    event::TransferProgressType,
    message::{Message, MessageType, Transfer},
    signing::{GenerateAddressMetadata, SignerType},
    storage::{MessageIndexation, MessageQueryFilter},
};
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
use crate::{client::ClientOptionsBuilder, event::emit_ledger_address_generation};

use iota_client::NodeInfoWrapper;

use chrono::prelude::{DateTime, Local};
use getset::{Getters, Setters};
use iota_client::bee_message::prelude::MessageId;
use serde::{Deserialize, Deserializer, Serialize};
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};

use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

mod sync;
pub(crate) use sync::{AccountSynchronizeStep, SyncedAccountData};
pub use sync::{AccountSynchronizer, SyncedAccount};

const ACCOUNT_ID_PREFIX: &str = "wallet-account://";

/// The account identifier.
#[derive(Debug, Clone, Serialize, Eq)]
#[serde(untagged)]
pub enum AccountIdentifier {
    /// An address identifier.
    #[serde(with = "crate::serde::iota_address_serde")]
    Address(AddressWrapper),
    /// A string identifier.
    Id(String),
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(usize),
}

impl<'de> Deserialize<'de> for AccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(AccountIdentifier::from(s))
    }
}

impl Hash for AccountIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Address(address) => address.to_bech32().hash(state),
            Self::Id(value) => value.hash(state),
            Self::Alias(value) => value.hash(state),
            Self::Index(i) => i.hash(state),
        }
    }
}

impl PartialEq for AccountIdentifier {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Address(address1), Self::Address(address2)) => address1 == address2,
            (Self::Id(id1), Self::Id(id2)) => id1 == id2,
            (Self::Alias(alias1), Self::Alias(alias2)) => alias1 == alias2,
            (Self::Index(index1), Self::Index(index2)) => index1 == index2,
            _ => false,
        }
    }
}

// When the identifier is a string id.
impl From<&str> for AccountIdentifier {
    fn from(value: &str) -> Self {
        if let Ok(address) = crate::address::parse(value) {
            Self::Address(address)
        } else if value.starts_with(ACCOUNT_ID_PREFIX) {
            Self::Id(value.to_string())
        } else {
            Self::Alias(value.to_string())
        }
    }
}

impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for AccountIdentifier {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

// When the identifier is an index.
impl From<usize> for AccountIdentifier {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

/// Account initialiser.
pub struct AccountInitialiser {
    accounts: AccountStore,
    storage_path: PathBuf,
    account_options: AccountOptions,
    sync_accounts_lock: Arc<Mutex<()>>,
    alias: Option<String>,
    created_at: Option<DateTime<Local>>,
    messages: Vec<Message>,
    addresses: Vec<Address>,
    #[doc(hidden)]
    pub client_options: ClientOptions,
    signer_type: Option<SignerType>,
    skip_persistence: bool,
    index: Option<usize>,
    allow_create_multiple_empty_accounts: bool,
}

impl AccountInitialiser {
    /// Initialises the account builder.
    pub(crate) fn new(
        client_options: ClientOptions,
        accounts: AccountStore,
        storage_path: PathBuf,
        account_options: AccountOptions,
        sync_accounts_lock: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            accounts,
            storage_path,
            account_options,
            sync_accounts_lock,
            alias: None,
            created_at: None,
            messages: vec![],
            addresses: vec![],
            client_options,
            #[cfg(feature = "stronghold")]
            signer_type: Some(SignerType::Stronghold),
            #[cfg(not(feature = "stronghold"))]
            signer_type: None,
            skip_persistence: false,
            index: None,
            allow_create_multiple_empty_accounts: false,
        }
    }

    /// Sets the account type.
    pub fn signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type.replace(signer_type);
        self
    }

    /// Defines the account alias. If not defined, we'll generate one.
    pub fn alias(mut self, alias: impl AsRef<str>) -> Self {
        self.alias.replace(alias.as_ref().to_string());
        self
    }

    /// Time of account creation.
    pub fn created_at(mut self, created_at: DateTime<Local>) -> Self {
        self.created_at.replace(created_at);
        self
    }

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    // Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    pub fn addresses(mut self, addresses: Vec<Address>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Skips storing the account to the database.
    pub fn skip_persistence(mut self) -> Self {
        self.skip_persistence = true;
        self
    }

    /// Sets the account index. Useful for account discovery.
    pub(crate) fn index(mut self, index: usize) -> Self {
        self.index.replace(index);
        self
    }

    /// Enables creating multiple accounts without history.
    /// The wallet disables it by default to simplify account discovery.
    pub fn allow_create_multiple_empty_accounts(mut self) -> Self {
        self.allow_create_multiple_empty_accounts = true;
        self
    }

    /// Initialises the account.
    pub async fn initialise(mut self) -> crate::Result<AccountHandle> {
        let signer_type = self.signer_type.ok_or(crate::Error::AccountInitialiseRequiredField(
            crate::error::AccountInitialiseRequiredField::SignerType,
        ))?;

        let index = if let Some(index) = self.index {
            index
        } else {
            let mut account_index = 0;
            for account in self.accounts.read().await.values() {
                if account.read().await.signer_type() == &signer_type {
                    account_index += 1;
                }
            }
            account_index
        };

        let alias = self.alias.unwrap_or_else(|| format!("Account {}", index + 1));
        let created_at = self.created_at.unwrap_or_else(Local::now);

        let mut latest_account_handle: Option<AccountHandle> = None;
        let mut latest_account_index = 0;
        for account_handle in self.accounts.read().await.values() {
            let account = account_handle.read().await;
            if account.alias() == &alias {
                return Err(crate::Error::AccountAliasAlreadyExists);
            }
            if *account.index() >= latest_account_index {
                latest_account_index = *account.index();
                latest_account_handle.replace(account_handle.clone());
            }
        }
        if !self.account_options.allow_create_multiple_empty_accounts && !self.allow_create_multiple_empty_accounts {
            if let Some(ref latest_account_handle) = latest_account_handle {
                let latest_account = latest_account_handle.read().await;
                if latest_account.with_messages(|messages| messages.is_empty()).await
                    && latest_account.addresses().iter().all(|a| a.outputs.is_empty())
                {
                    return Err(crate::Error::LatestAccountIsEmpty);
                }
            }
        }

        self.addresses.sort();

        let mut account = Account {
            id: index.to_string(),
            signer_type: signer_type.clone(),
            index,
            alias,
            created_at,
            last_synced_at: None,
            addresses: self.addresses,
            client_options: self.client_options,
            storage_path: self.storage_path.clone(),
            skip_persistence: self.skip_persistence,
            cached_messages: Default::default(),
        };

        let bech32_hrp = match account.client_options.network().as_deref() {
            Some("testnet") => "atoi".to_string(),
            Some("mainnet") => "iota".to_string(),
            Some("chrysalis-mainnet") => "iota".to_string(),
            _ => {
                let client_options = account.client_options.clone();
                let get_from_client_task = async {
                    let hrp = crate::client::get_client(&client_options)
                        .await?
                        .read()
                        .await
                        .get_network_info()
                        .await
                        .map_err(|e| match e {
                            iota_client::Error::SyncedNodePoolEmpty => crate::Error::NodesNotSynced(
                                client_options
                                    .nodes()
                                    .iter()
                                    .map(|node| node.url.as_str())
                                    .collect::<Vec<&str>>()
                                    .join(", "),
                            ),
                            _ => e.into(),
                        })?
                        .bech32_hrp;
                    crate::Result::Ok(hrp)
                };
                match latest_account_handle {
                    Some(handle) => {
                        let latest_account = handle.read().await;
                        if latest_account.client_options == account.client_options {
                            latest_account.bech32_hrp()
                        } else {
                            get_from_client_task.await?
                        }
                    }
                    None => get_from_client_task.await?,
                }
            }
        };

        for address in account.addresses.iter_mut() {
            address.set_bech32_hrp(bech32_hrp.to_string());
        }
        for message in self.messages.iter_mut() {
            message.set_bech32_hrp(bech32_hrp.to_string());
        }

        let address = match account.addresses.first() {
            Some(address) => address.address().clone(),
            None => {
                let network = match bech32_hrp.as_ref() {
                    "iota" => crate::signing::Network::Mainnet,
                    _ => crate::signing::Network::Testnet,
                };
                let address = crate::address::get_iota_address(
                    &account,
                    0,
                    false,
                    bech32_hrp,
                    // We set it to syncing: true so it will not be shown on the ledger
                    GenerateAddressMetadata { syncing: true, network },
                )
                .await?;

                account.addresses.push(
                    AddressBuilder::new()
                        .address(address.clone())
                        .key_index(0)
                        .internal(false)
                        .outputs(Vec::new())
                        .build()
                        .unwrap(), // safe to unwrap since we provide all required fields
                );
                address
            }
        };
        // store first address for the first ledger account so we can verify that the correct mnemonic is used when a
        // new account is created
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        if account.index == 0 {
            match signer_type {
                #[cfg(feature = "ledger-nano")]
                SignerType::LedgerNano => {
                    crate::storage::get(&self.storage_path)
                        .await?
                        .lock()
                        .await
                        .save_first_ledger_address(&address.inner)
                        .await?;
                    log::debug!("[LEDGERADDRESS] saved first address {:?}", &address.inner);
                }
                #[cfg(feature = "ledger-nano-simulator")]
                SignerType::LedgerNanoSimulator => {
                    crate::storage::get(&self.storage_path)
                        .await?
                        .lock()
                        .await
                        .save_first_ledger_address(&address.inner)
                        .await?;
                    log::debug!("[LEDGERADDRESS] saved first address {:?}", &address.inner);
                }
                _ => {}
            }
        } else {
            match signer_type {
                #[cfg(feature = "ledger-nano")]
                SignerType::LedgerNano => {
                    let signer = crate::signing::get_signer(&SignerType::LedgerNano).await;
                    let mut signer = signer.lock().await;
                    let first_address = signer
                        // dummy account with index 0 so we can generate the correct address
                        .generate_address(
                            &Account {
                                id: "account_for_first_ledger_address".to_string(),
                                signer_type: signer_type.clone(),
                                index: 0,
                                alias: "account_for_first_ledger_address".to_string(),
                                created_at: Local::now(),
                                last_synced_at: None,
                                addresses: vec![],
                                client_options: ClientOptionsBuilder::new().build()?,
                                storage_path: PathBuf::new(),
                                skip_persistence: true,
                                cached_messages: Arc::new(Mutex::new(HashMap::new())),
                            },
                            0,
                            false,
                            GenerateAddressMetadata {
                                syncing: true,
                                network: account.network(),
                            },
                        )
                        .await?;
                    log::debug!("[LEDGERADDRESS] generated first address {:?}", first_address);
                    // generate address from first account to validate mnemonic
                    if let Ok(first_account_first_address) = crate::storage::get(&self.storage_path)
                        .await?
                        .lock()
                        .await
                        .get_first_ledger_address()
                        .await
                    {
                        log::debug!("[LEDGERADDRESS] read first address {:?}", first_account_first_address);
                        if first_account_first_address != first_address {
                            return Err(crate::Error::LedgerMnemonicMismatch);
                        }
                    }
                }
                #[cfg(feature = "ledger-nano-simulator")]
                SignerType::LedgerNanoSimulator =>
                // generate address from first account to validate mnemonic
                {
                    let signer = crate::signing::get_signer(&SignerType::LedgerNanoSimulator).await;
                    let mut signer = signer.lock().await;
                    let first_address = signer
                        // dummy account with index 0 so we can generate the correct address
                        .generate_address(
                            &Account {
                                id: "account_for_first_ledger_address".to_string(),
                                signer_type: signer_type.clone(),
                                index: 0,
                                alias: "account_for_first_ledger_address".to_string(),
                                created_at: Local::now(),
                                last_synced_at: None,
                                addresses: vec![],
                                client_options: ClientOptionsBuilder::new().build()?,
                                storage_path: PathBuf::new(),
                                skip_persistence: true,
                                cached_messages: Arc::new(Mutex::new(HashMap::new())),
                            },
                            0,
                            false,
                            GenerateAddressMetadata {
                                syncing: true,
                                network: account.network(),
                            },
                        )
                        .await?;
                    log::debug!("[LEDGERADDRESS] generated first address {:?}", first_address);
                    if let Ok(first_account_first_address) = crate::storage::get(&self.storage_path)
                        .await?
                        .lock()
                        .await
                        .get_first_ledger_address()
                        .await
                    {
                        log::debug!("[LEDGERADDRESS] read first address {:?}", first_account_first_address);
                        if first_account_first_address != first_address {
                            return Err(crate::Error::LedgerMnemonicMismatch);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut digest = [0; 32];
        let raw = match address.as_ref() {
            iota_client::bee_message::address::Address::Ed25519(a) => a.as_ref().to_vec(),
        };
        crypto::hashes::sha::SHA256(&raw, &mut digest);
        account.set_id(format!("{}{}", ACCOUNT_ID_PREFIX, hex::encode(digest)));

        let guard = if self.skip_persistence {
            AccountHandle::new(
                account,
                self.accounts.clone(),
                self.account_options,
                self.sync_accounts_lock.clone(),
            )
        } else {
            account.save().await?;
            if !self.messages.is_empty() {
                account.save_messages(self.messages).await?;
            }
            let account_id = account.id().clone();
            let guard = AccountHandle::new(
                account,
                self.accounts.clone(),
                self.account_options,
                self.sync_accounts_lock.clone(),
            );
            self.accounts.write().await.insert(account_id, guard.clone());
            // monitor on a non-async function to prevent cycle computing the `monitor_address_balance` fn type
            monitor_address(guard.clone());
            guard
        };

        Ok(guard)
    }
}

fn monitor_address(account_handle: AccountHandle) {
    crate::spawn(async move {
        // ignore errors because we fallback to the polling system
        let _ = crate::monitor::monitor_account_addresses_balance(account_handle).await;
    });
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    #[getset(set = "pub(crate)")]
    id: String,
    /// The account's signer type.
    #[serde(rename = "signerType")]
    signer_type: SignerType,
    /// The account index
    index: usize,
    /// The account alias.
    alias: String,
    /// Time of account creation.
    #[serde(rename = "createdAt")]
    created_at: DateTime<Local>,
    /// Time the account was last synced with the Tangle.
    #[serde(rename = "lastSyncedAt")]
    #[getset(set = "pub(crate)")]
    last_synced_at: Option<DateTime<Local>>,
    /// Address history associated with the seed.
    #[getset(set = "pub(crate)")]
    addresses: Vec<Address>,
    /// The client options.
    #[serde(rename = "clientOptions")]
    client_options: ClientOptions,
    #[getset(set = "pub(crate)", get = "pub(crate)")]
    #[serde(rename = "storagePath")]
    storage_path: PathBuf,
    #[getset(set = "pub(crate)", get = "pub(crate)")]
    #[serde(skip)]
    skip_persistence: bool,
    #[getset(get = "pub(crate)")]
    #[serde(skip)]
    pub(crate) cached_messages: Arc<Mutex<HashMap<MessageId, Message>>>,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// A thread guard over an account.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    inner: Arc<RwLock<Account>>,
    pub(crate) accounts: AccountStore,
    pub(crate) locked_outputs: Arc<Mutex<Vec<AddressOutput>>>,
    pub(crate) account_options: AccountOptions,
    is_mqtt_enabled: Arc<AtomicBool>,
    pub(crate) change_addresses_to_sync: Arc<Mutex<HashSet<AddressWrapper>>>,
    pub(crate) sync_accounts_lock: Arc<Mutex<()>>,
}

impl AccountHandle {
    pub(crate) fn new(
        account: Account,
        accounts: AccountStore,
        account_options: AccountOptions,
        sync_accounts_lock: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(account)),
            accounts,
            locked_outputs: Default::default(),
            account_options,
            is_mqtt_enabled: Arc::new(AtomicBool::new(true)),
            change_addresses_to_sync: Default::default(),
            sync_accounts_lock,
        }
    }

    /// Returns the addresses that need output consolidation.
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    pub(crate) async fn output_consolidation_addresses(&self) -> crate::Result<Vec<AddressWrapper>> {
        let mut addresses = Vec::new();
        let account = self.inner.read().await;
        let sent_messages = account.list_messages(0, 0, Some(MessageType::Sent)).await?;
        for address in account.addresses() {
            if address.available_outputs(&sent_messages).len() >= self.account_options.output_consolidation_threshold {
                addresses.push(address.address().clone());
            }
        }
        Ok(addresses)
    }

    pub(crate) fn is_mqtt_enabled(&self) -> bool {
        self.is_mqtt_enabled.load(Ordering::Relaxed)
    }

    pub(crate) fn disable_mqtt(&self) {
        self.is_mqtt_enabled.store(false, Ordering::Relaxed);
    }

    pub(crate) fn enable_mqtt(&self) {
        self.is_mqtt_enabled.store(true, Ordering::Relaxed);
    }
}

impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

macro_rules! guard_field_getters {
    ($ty:ident, $(#[$attr:meta] => $x:ident => $ret:ty),*) => {
        impl $ty {
            $(
                #[$attr]
                pub async fn $x(&self) -> $ret {
                    self.inner.read().await.$x().clone()
                }
            )*
        }
    }
}

guard_field_getters!(
    AccountHandle,
    #[doc = "Bridge to [Account#id](struct.Account.html#method.id)."] => id => String,
    #[doc = "Bridge to [Account#signer_type](struct.Account.html#method.signer_type)."] => signer_type => SignerType,
    #[doc = "Bridge to [Account#index](struct.Account.html#method.index)."] => index => usize,
    #[doc = "Bridge to [Account#alias](struct.Account.html#method.alias)."] => alias => String,
    #[doc = "Bridge to [Account#created_at](struct.Account.html#method.created_at)."] => created_at => DateTime<Local>,
    #[doc = "Bridge to [Account#last_synced_at](struct.Account.html#method.last_synced_at)."] => last_synced_at => Option<DateTime<Local>>,
    #[doc = "Bridge to [Account#addresses](struct.Account.html#method.addresses).
    This method clones the addresses so prefer the using the `read` method to access the account instance."] => addresses => Vec<Address>,
    #[doc = "Bridge to [Account#client_options](struct.Account.html#method.client_options)."] => client_options => ClientOptions,
    #[doc = "Bridge to [Account#bech32_hrp](struct.Account.html#method.bech32_hrp)."] => bech32_hrp => String
);

impl AccountHandle {
    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub async fn sync(&self) -> AccountSynchronizer {
        AccountSynchronizer::new(self.clone()).await
    }

    async fn sync_internal(&self) -> AccountSynchronizer {
        AccountSynchronizer::new(self.clone()).await.skip_change_addresses()
    }

    /// Consolidate account outputs.
    pub async fn consolidate_outputs(&self, include_dust_allowance_outputs: bool) -> crate::Result<Vec<Message>> {
        self.sync_internal()
            .await
            .execute()
            .await?
            .consolidate_outputs(include_dust_allowance_outputs)
            .await
    }

    /// Send messages.
    pub async fn transfer(&self, transfer_obj: Transfer) -> crate::Result<Message> {
        let account_id = self.id().await;
        let synced = if transfer_obj.skip_sync {
            SyncedAccount::from(self.clone()).await
        } else {
            transfer_obj
                .emit_event_if_needed(account_id.clone(), TransferProgressType::SyncingAccount)
                .await;
            self.sync_internal().await.execute().await?
        };
        synced.transfer(transfer_obj).await
    }

    /// Retry message.
    pub async fn retry(&self, message_id: &MessageId) -> crate::Result<Message> {
        self.sync_internal().await.execute().await?.retry(message_id).await
    }

    /// Promote message.
    pub async fn promote(&self, message_id: &MessageId) -> crate::Result<Message> {
        self.sync_internal().await.execute().await?.promote(message_id).await
    }

    /// Reattach message.
    pub async fn reattach(&self, message_id: &MessageId) -> crate::Result<Message> {
        self.sync_internal().await.execute().await?.reattach(message_id).await
    }

    /// Gets a new unused address and links it to this account.
    pub async fn generate_address(&self) -> crate::Result<Address> {
        let mut account = self.inner.write().await;
        self.generate_address_internal(&mut account).await
    }

    /// Generates an address without locking the account.
    pub(crate) async fn generate_address_internal(
        &self,
        account: &mut RwLockWriteGuard<'_, Account>,
    ) -> crate::Result<Address> {
        let address = crate::address::get_new_address(
            account,
            GenerateAddressMetadata {
                syncing: false,
                network: account.network(),
            },
        )
        .await?;

        account
            .do_mut(|account| {
                account.addresses.push(address.clone());
                Ok(())
            })
            .await?;

        // monitor on a non-async function to prevent cycle computing the `monitor_address_balance` fn type
        self.monitor_address(address.address().clone());

        Ok(address)
    }

    /// Gets amount new unused addresses and links them to this account.
    pub async fn generate_addresses(&self, amount: usize) -> crate::Result<Vec<Address>> {
        let mut account = self.inner.write().await;
        self.generate_addresses_internal(&mut account, amount).await
    }

    /// Generates an address without locking the account.
    pub(crate) async fn generate_addresses_internal(
        &self,
        account: &mut RwLockWriteGuard<'_, Account>,
        amount: usize,
    ) -> crate::Result<Vec<Address>> {
        let key_index = account.addresses().iter().filter(|a| !a.internal()).count();
        let bech32_hrp = match account.addresses().first() {
            Some(address) => address.address().bech32_hrp().to_string(),
            None => {
                crate::client::get_client(account.client_options())
                    .await?
                    .read()
                    .await
                    .get_network_info()
                    .await?
                    .bech32_hrp
            }
        };

        let mut addresses = Vec::new();
        for key_index in key_index..amount + key_index {
            addresses.push(
                crate::address::get_address_with_index(
                    account,
                    key_index,
                    bech32_hrp.clone(),
                    GenerateAddressMetadata {
                        syncing: false,
                        network: account.network(),
                    },
                )
                .await?,
            );
        }

        account
            .do_mut(|account| {
                account.addresses.extend(addresses.clone());
                Ok(())
            })
            .await?;

        // Don't monitor if too many addresses
        if addresses.len() < 1000 {
            for address in &addresses {
                // monitor on a non-async function to prevent cycle computing the `monitor_address_balance` fn type
                self.monitor_address(address.address().clone());
            }
        }

        Ok(addresses)
    }

    fn monitor_address(&self, address: AddressWrapper) {
        let handle = self.clone();
        crate::spawn(async move {
            // ignore errors because we fallback to the polling system
            let _ = crate::monitor::monitor_address_balance(handle, vec![address]).await;
        });
    }

    /// Synchronizes the account addresses with the Tangle and returns the latest address in the account,
    /// which is an address without balance.
    pub async fn get_unused_address(&self) -> crate::Result<Address> {
        self.sync_internal()
            .await
            .steps(vec![AccountSynchronizeStep::SyncAddresses(None)])
            .execute()
            .await?;
        // safe to clone since the `sync` guarantees a latest unused address
        let address = self.latest_address().await;
        // regenerate address for ledger accounts
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        {
            let account = self.read().await;
            let ledger = match account.signer_type() {
                #[cfg(feature = "ledger-nano")]
                SignerType::LedgerNano => true,
                #[cfg(feature = "ledger-nano-simulator")]
                SignerType::LedgerNanoSimulator => true,
                _ => false,
            };
            if ledger {
                // Send address event so it can be displayed before and then compared with the prompt on the ledger
                emit_ledger_address_generation(&account, address.address().to_bech32()).await;

                log::debug!("get_unused_address regenerate address so it's displayed on the ledger");
                let regenerated_address = crate::address::get_address_with_index(
                    &account,
                    *address.key_index(),
                    account.bech32_hrp(),
                    GenerateAddressMetadata {
                        syncing: false,
                        network: account.network(),
                    },
                )
                .await?;
                if address.address().inner != regenerated_address.address().inner {
                    return Err(crate::Error::LedgerMnemonicMismatch);
                }
            }
        }
        Ok(address)
    }

    /// Syncs the latest address with the Tangle and determines whether it's unused or not.
    /// An unused address is an address without balance and associated message history.
    /// Note that such address might have been used in the past, because the message history might have been pruned by
    /// the node.
    pub async fn is_latest_address_unused(&self) -> crate::Result<bool> {
        let mut account = self.inner.write().await;
        let client_options = account.client_options().clone();
        let messages: Vec<(MessageId, Option<bool>)> = account
            .with_messages(|messages| messages.iter().map(|m| (m.key, m.confirmed)).collect())
            .await;
        let latest_address = account.latest_address_mut();
        let bech32_hrp = latest_address.address().bech32_hrp().to_string();
        let address_wrapper = latest_address.address().clone();
        sync::sync_address(
            messages,
            &client_options,
            latest_address.outputs_mut(),
            address_wrapper,
            bech32_hrp,
            self.account_options,
        )
        .await?;
        let is_unused = latest_address.balance() == 0 && latest_address.outputs().is_empty();
        account.save().await?;
        Ok(is_unused)
    }

    /// Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
    pub async fn latest_address(&self) -> Address {
        self.inner.read().await.latest_address().clone()
    }

    /// Bridge to [Account#balance](struct.Account.html#method.balance).
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        self.inner.read().await.balance().await
    }

    /// Bridge to [Account#set_alias](struct.Account.html#method.set_alias).
    pub async fn set_alias(&self, alias: impl AsRef<str>) -> crate::Result<()> {
        self.inner.write().await.set_alias(alias).await
    }

    /// Bridge to [Account#set_client_options](struct.Account.html#method.set_client_options).
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        self.inner.write().await.set_client_options(options).await
    }

    /// Bridge to [Account#list_messages](struct.Account.html#method.list_messages).
    /// This method clones the account's messages so when querying a large list of messages
    /// prefer using the `read` method to access the account instance.
    pub async fn list_messages(
        &self,
        count: usize,
        from: usize,
        message_type: Option<MessageType>,
    ) -> crate::Result<Vec<Message>> {
        self.inner.read().await.list_messages(count, from, message_type).await
    }

    /// Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_spent_addresses(&self) -> crate::Result<Vec<Address>> {
        Ok(self
            .inner
            .read()
            .await
            .list_spent_addresses()
            .await?
            .into_iter()
            .cloned()
            .collect())
    }

    /// Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_unspent_addresses(&self) -> crate::Result<Vec<Address>> {
        Ok(self
            .inner
            .read()
            .await
            .list_unspent_addresses()
            .await?
            .into_iter()
            .cloned()
            .collect())
    }

    /// Bridge to [Account#get_message](struct.Account.html#method.get_message).
    pub async fn get_message(&self, message_id: &MessageId) -> Option<Message> {
        self.inner.read().await.get_message(message_id).await
    }

    /// Bridge to [Account#get_node_info](struct.Account.html#method.get_node_info).
    pub async fn get_node_info(
        &self,
        url: Option<&str>,
        jwt: Option<&str>,
        auth: Option<(&str, &str)>,
    ) -> crate::Result<NodeInfoWrapper> {
        self.inner.read().await.get_node_info(url, jwt, auth).await
    }
}

/// Account balance information.
#[derive(Debug, Serialize)]
pub struct AccountBalance {
    /// Account's total balance.
    pub total: u64,
    // The available balance is the balance users are allowed to spend.
    /// For example, if a user with 50i total account balance has made a message spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub available: u64,
    /// Balances from message with `incoming: true`.
    /// Note that this may not be accurate since the node prunes the messags.
    pub incoming: u64,
    /// Balances from message with `incoming: false`.
    /// Note that this may not be accurate since the node prunes the messags.
    pub outgoing: u64,
}

impl Account {
    pub(crate) async fn save(&mut self) -> crate::Result<()> {
        if !self.skip_persistence {
            let storage_path = self.storage_path.clone();
            crate::storage::get(&storage_path)
                .await?
                .lock()
                .await
                .save_account(&self.id, self)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn do_mut<R>(&mut self, f: impl FnOnce(&mut Self) -> crate::Result<R>) -> crate::Result<R> {
        let res = f(self)?;
        self.save().await?;
        Ok(res)
    }

    /// Do something with the indexed messages.
    pub(crate) async fn with_messages<T, C: FnOnce(&Vec<MessageIndexation>) -> T>(&self, f: C) -> T {
        if self.skip_persistence {
            f(&Vec::new())
        } else {
            f(crate::storage::get(&self.storage_path)
                .await
                .expect("storage adapter not set")
                .lock()
                .await
                .message_indexation(self)
                .expect("message indexation not set"))
        }
    }

    /// Returns the address bech32 human readable part.
    pub fn bech32_hrp(&self) -> String {
        self.addresses().first().unwrap().address().bech32_hrp().to_string()
    }

    /// Returns the address bech32 human readable part.
    pub(crate) fn network(&self) -> crate::signing::Network {
        match self.addresses().first().unwrap().address().bech32_hrp() {
            "iota" => crate::signing::Network::Mainnet,
            _ => crate::signing::Network::Testnet,
        }
    }

    /// Returns the most recent address of the account.
    pub fn latest_address(&self) -> &Address {
        // the addresses list is never empty because we generate an address on the account creation
        self.addresses
            .iter()
            .filter(|a| !a.internal())
            .max_by_key(|a| a.key_index())
            .unwrap()
    }

    /// Returns the most recent change address of the account.
    pub(crate) fn latest_change_address(&self) -> Option<&Address> {
        self.addresses
            .iter()
            .filter(|a| *a.internal())
            .max_by_key(|a| a.key_index())
    }

    fn latest_address_mut(&mut self) -> &mut Address {
        // the addresses list is never empty because we generate an address on the account creation
        self.addresses
            .iter_mut()
            .filter(|a| !a.internal())
            .max_by_key(|a| *a.key_index())
            .unwrap()
    }

    pub(crate) async fn balance_internal(&self, sent_messages: &[Message]) -> AccountBalance {
        let (incoming, outgoing) = self
            .with_messages(|messages| {
                messages.iter().filter(|m| m.confirmed == Some(true)).fold(
                    (0, 0),
                    |(incoming, outgoing), message: &MessageIndexation| {
                        if message.internal == Some(false) {
                            if message.incoming.unwrap_or(false) {
                                return (incoming + message.value, outgoing);
                            } else {
                                return (incoming, outgoing + message.value);
                            }
                        }
                        (incoming, outgoing)
                    },
                )
            })
            .await;

        AccountBalance {
            total: self.addresses.iter().fold(0, |acc, address| acc + address.balance()),
            available: self
                .addresses()
                .iter()
                .fold(0, |acc, addr| acc + addr.available_balance(sent_messages)),
            incoming,
            outgoing,
        }
    }

    /// Gets the account balance information.
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        let sent_messages = self.list_messages(0, 0, Some(MessageType::Sent)).await?;
        Ok(self.balance_internal(&sent_messages).await)
    }

    /// Updates the account alias.
    pub async fn set_alias(&mut self, alias: impl AsRef<str>) -> crate::Result<()> {
        let alias = alias.as_ref().to_string();

        self.alias = alias;
        self.save().await
    }

    /// Updates the account's client options.
    pub async fn set_client_options(&mut self, options: ClientOptions) -> crate::Result<()> {
        let client_guard = crate::client::get_client(&options).await?;
        let client = client_guard.read().await;

        let unsynced_nodes = client.unsynced_nodes().await;
        if !unsynced_nodes.is_empty() {
            let diff_nodes: Vec<&Node> = options
                .nodes()
                .iter()
                .filter(|node| !self.client_options.nodes().contains(node))
                .collect();
            let unsynced_diff_nodes: Vec<&str> = unsynced_nodes
                .into_iter()
                .filter(|url| diff_nodes.iter().any(|node| node.url == url.url))
                .map(|url| url.url.as_str())
                .collect();
            if !unsynced_diff_nodes.is_empty() {
                return Err(crate::Error::NodesNotSynced(unsynced_diff_nodes.join(", ")));
            }
        }

        let bech32_hrp = client.get_network_info().await?.bech32_hrp;
        for address in &mut self.addresses {
            address.set_bech32_hrp(bech32_hrp.to_string());
        }

        self.client_options = options;

        self.save().await
    }

    /// Gets a list of transactions on this account.
    /// It's fetched from the storage. To ensure the database is updated with the latest transactions,
    /// `sync` should be called first.
    ///
    /// * `count` - Number of (most recent) messages to fetch.
    /// * `from` - Starting point of the subset to fetch.
    /// * `message_type` - Optional message type filter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use iota_wallet::{
    ///     account_manager::AccountManager, client::ClientOptionsBuilder, message::MessageType, signing::SignerType,
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     // gets 10 received messages, skipping the first 5 most recent messages.
    ///     let client_options = ClientOptionsBuilder::new()
    ///         .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
    ///         .expect("invalid node URL")
    ///         .build()
    ///         .unwrap();
    ///     let mut manager = AccountManager::builder()
    ///         .with_storage("./test-storage", None)
    ///         .unwrap()
    ///         .finish()
    ///         .await
    ///         .unwrap();
    ///     manager.set_stronghold_password("password").await.unwrap();
    ///     manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();
    ///
    ///     let account_handle = manager
    ///         .create_account(client_options)
    ///         .unwrap()
    ///         .initialise()
    ///         .await
    ///         .expect("failed to add account");
    ///     let account = account_handle.read().await;
    ///     account.list_messages(10, 5, Some(MessageType::Received));
    /// }
    /// ```
    pub async fn list_messages(
        &self,
        count: usize,
        from: usize,
        message_type: Option<MessageType>,
    ) -> crate::Result<Vec<Message>> {
        let mut messages = crate::storage::get(&self.storage_path)
            .await
            .expect("storage adapter not set")
            .lock()
            .await
            .get_messages(
                self,
                count,
                from,
                MessageQueryFilter::message_type(message_type.clone()),
            )
            .await?;

        messages.sort_unstable_by(|a, b| a.timestamp().cmp(b.timestamp()));

        Ok(messages)
    }

    /// Gets the spent addresses.
    pub async fn list_spent_addresses(&self) -> crate::Result<Vec<&Address>> {
        let sent_messages = self.list_messages(0, 0, Some(MessageType::Sent)).await?;
        let spent_addresses = self
            .addresses
            .iter()
            .filter(|address| !crate::address::is_unspent(&sent_messages, address.address()))
            .collect();
        Ok(spent_addresses)
    }

    /// Gets the spent addresses.
    pub async fn list_unspent_addresses(&self) -> crate::Result<Vec<&Address>> {
        let sent_messages = self.list_messages(0, 0, Some(MessageType::Sent)).await?;
        let unspent_addresses = self
            .addresses
            .iter()
            .filter(|address| crate::address::is_unspent(&sent_messages, address.address()))
            .collect();
        Ok(unspent_addresses)
    }

    pub(crate) fn append_addresses(&mut self, addresses: Vec<Address>) {
        addresses
            .into_iter()
            .for_each(|address| match self.addresses.iter().position(|a| a == &address) {
                Some(index) => {
                    self.addresses[index] = address;
                }
                None => {
                    self.addresses.push(address);
                }
            });
    }

    /// Gets the node info from /api/v1/info endpoint
    // TODO: Add auth and url in one NodeInfoOptions struct.
    pub async fn get_node_info(
        &self,
        url: Option<&str>,
        jwt: Option<&str>,
        auth: Option<(&str, &str)>,
    ) -> crate::Result<NodeInfoWrapper> {
        let info = match url {
            Some(url) => NodeInfoWrapper {
                nodeinfo: iota_client::Client::get_node_info(url, jwt.map(Into::into), auth)
                    .await
                    .map_err(|e| crate::Error::ClientError(Box::new(e)))?,
                url: url.to_string(),
            },
            None => {
                let client_guard = crate::client::get_client(self.client_options()).await?;
                let client = client_guard.read().await;

                client
                    .get_info()
                    .await
                    .map_err(|e| crate::Error::ClientError(Box::new(e)))?
            }
        };

        Ok(info)
    }

    #[cfg(test)]
    pub(crate) fn addresses_mut(&mut self) -> &mut Vec<Address> {
        &mut self.addresses
    }

    pub(crate) async fn save_messages(&mut self, messages: Vec<Message>) -> crate::Result<()> {
        crate::storage::get(&self.storage_path)
            .await?
            .lock()
            .await
            .save_messages(self, &messages)
            .await
    }

    /// Gets a message with the given id associated with this account.
    pub async fn get_message(&self, message_id: &MessageId) -> Option<Message> {
        crate::storage::get(&self.storage_path)
            .await
            .expect("storage adapter not set")
            .lock()
            .await
            .get_message(self, message_id)
            .await
            .ok()
    }

    /// Gets the available balance on the given address.
    pub async fn address_available_balance(&self, address: &Address) -> crate::Result<u64> {
        Ok(address.available_balance(&self.list_messages(0, 0, Some(MessageType::Sent)).await?))
    }
}

#[cfg(test)]
mod tests {
    use super::AccountHandle;
    use crate::{
        account_manager::AccountManager,
        address::{Address, AddressBuilder, AddressOutput, OutputKind},
        client::ClientOptionsBuilder,
        message::{Message, MessagePayload, MessageType, TransactionEssence},
    };
    use iota_client::bee_message::prelude::{MessageId, TransactionId};
    use std::collections::HashMap;

    // asserts that the `set_alias` function updates the account alias in storage
    #[tokio::test]
    async fn set_alias() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

            let updated_alias = "updated alias";

            account_handle.set_alias(updated_alias).await.unwrap();

            let account_in_storage = manager
                .get_account(account_handle.read().await.id())
                .await
                .expect("failed to get account from storage");
            assert_eq!(account_in_storage.alias().await, updated_alias.to_string());
        })
        .await;
    }

    // asserts that the `set_client_options` function updates the account client options in storage
    #[tokio::test]
    async fn set_client_options() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

            let updated_client_options = ClientOptionsBuilder::new()
                .with_nodes(&[
                    "https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe",
                    "https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe",
                ])
                .unwrap()
                .build()
                .unwrap();

            account_handle
                .set_client_options(updated_client_options.clone())
                .await
                .unwrap();

            let account_in_storage = manager
                .get_account(account_handle.read().await.id())
                .await
                .expect("failed to get account from storage");
            assert_eq!(account_in_storage.client_options().await, updated_client_options);
        })
        .await;
    }

    #[tokio::test]
    async fn account_handle_bridge_getters() {
        let manager = crate::test_utils::get_account_manager().await;

        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

        macro_rules! assert_bridge_method {
            ($($x:ident),+) => {
                $(
                    let result = account_handle.$x().await;
                    let expected = account_handle.read().await.$x().clone();
                    assert_eq!(result, expected);
                )*
            };
        }

        assert_bridge_method!(id, signer_type, index, alias, created_at, addresses, client_options);
    }

    fn _generate_address_output(value: u64) -> AddressOutput {
        let mut tx_id = [0; 32];
        crypto::utils::rand::fill(&mut tx_id).unwrap();
        AddressOutput {
            transaction_id: TransactionId::new(tx_id),
            message_id: MessageId::new([0; 32]),
            index: 0,
            amount: value,
            is_spent: false,
            address: crate::test_utils::generate_random_iota_address(),
            kind: OutputKind::SignatureLockedSingle,
        }
    }

    async fn _generate_account(manager: &AccountManager, messages: Vec<Message>) -> (AccountHandle, Address, u64) {
        let balance = 30;
        let first_address = AddressBuilder::new()
            .address(crate::test_utils::generate_random_iota_address())
            .key_index(0)
            .outputs(vec![_generate_address_output(balance / 2_u64)])
            .build()
            .unwrap();
        let second_address = AddressBuilder::new()
            .address(crate::test_utils::generate_random_iota_address())
            .key_index(1)
            .outputs(vec![_generate_address_output(balance / 2_u64)])
            .build()
            .unwrap();

        let addresses = vec![second_address.clone(), first_address];
        let account_handle = crate::test_utils::AccountCreator::new(manager)
            .addresses(addresses)
            .messages(messages)
            .create()
            .await;
        (account_handle, second_address, balance)
    }

    #[tokio::test]
    async fn generate_address() {
        crate::test_utils::with_account_manager(
            crate::test_utils::TestType::Signing,
            |manager, signer_type| async move {
                let account_handle = crate::test_utils::AccountCreator::new(&manager)
                    .signer_type(signer_type)
                    .create()
                    .await;
                let account_next_address = {
                    let account = account_handle.read().await;
                    crate::address::get_new_address(
                        &account,
                        crate::signing::GenerateAddressMetadata {
                            syncing: false,
                            network: account.network(),
                        },
                    )
                    .await
                    .unwrap()
                };
                let generated_address = account_handle.generate_address().await.unwrap();

                assert_eq!(generated_address, account_next_address);
                assert_eq!(account_handle.latest_address().await, generated_address);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn latest_address() {
        let manager = crate::test_utils::get_account_manager().await;
        let (account_handle, latest_address, _) = _generate_account(&manager, vec![]).await;
        assert_eq!(account_handle.read().await.latest_address(), &latest_address);
    }

    #[tokio::test]
    async fn total_balance() {
        let manager = crate::test_utils::get_account_manager().await;
        let (account_handle, _, balance) = _generate_account(&manager, vec![]).await;
        assert_eq!(account_handle.read().await.balance().await.unwrap().total, balance);
    }

    #[tokio::test]
    async fn available_balance() {
        let manager = crate::test_utils::get_account_manager().await;
        let (account_handle, _, balance) = _generate_account(&manager, vec![]).await;
        assert_eq!(account_handle.read().await.balance().await.unwrap().available, balance);

        let first_address = {
            let mut account = account_handle.write().await;
            let address = account.addresses_mut().iter_mut().next().unwrap();
            let mut outputs = HashMap::new();
            let output = _generate_address_output(15);
            outputs.insert(output.id().unwrap(), output);
            address.outputs = outputs;
            address.clone()
        };
        let second_address = {
            let mut account = account_handle.write().await;
            let addresses = account.addresses_mut();
            let mut iter = addresses.iter_mut();
            iter.next();
            let address = iter.next().unwrap();
            let mut outputs = HashMap::new();
            let output = _generate_address_output(15);
            outputs.insert(output.id().unwrap(), output);
            address.outputs = outputs;
            address.clone()
        };

        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(first_address.clone())
            .value(15)
            .input_transaction_id(first_address.outputs.values().next().unwrap().transaction_id)
            .input_address(Some(second_address.address().clone()))
            .account_addresses(account_handle.addresses().await)
            .confirmed(None)
            .build()
            .await;
        let confirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(second_address.clone())
            .value(10)
            .input_transaction_id(second_address.outputs.values().next().unwrap().transaction_id)
            .confirmed(Some(true))
            .build()
            .await;

        account_handle
            .write()
            .await
            .save_messages(vec![unconfirmed_message.clone(), confirmed_message])
            .await
            .unwrap();

        assert_eq!(
            account_handle.read().await.balance().await.unwrap().available,
            balance
                - if let Some(MessagePayload::Transaction(tx)) = unconfirmed_message.payload() {
                    let TransactionEssence::Regular(essence) = tx.essence();
                    essence.value()
                } else {
                    0
                }
        );
    }

    #[tokio::test]
    async fn list_all_messages() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager)
            .addresses(vec![crate::test_utils::generate_random_address()])
            .create()
            .await;
        let latest_address = account_handle.read().await.latest_address().clone();
        let received_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .input_address(Some(crate::test_utils::generate_random_iota_address()))
            .build()
            .await;
        let failed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .broadcasted(false)
            .build()
            .await;
        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .confirmed(None)
            .build()
            .await;
        let value_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .build()
            .await;
        account_handle
            .write()
            .await
            .save_messages(vec![
                received_message,
                failed_message,
                unconfirmed_message,
                value_message,
            ])
            .await
            .unwrap();

        let txs = account_handle.list_messages(4, 0, None).await.unwrap();
        assert_eq!(txs.len(), 4);
    }

    #[tokio::test]
    async fn list_messages_by_type() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager)
            .addresses(vec![crate::test_utils::generate_random_address()])
            .create()
            .await;

        let external_address = crate::test_utils::generate_random_address();
        let latest_address = account_handle.read().await.latest_address().clone();

        let received_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .input_address(Some(external_address.address().clone()))
            .confirmed(Some(true))
            .broadcasted(true)
            .build()
            .await;
        let sent_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(external_address.clone())
            .input_address(Some(latest_address.address().clone()))
            .account_addresses(account_handle.addresses().await)
            .confirmed(Some(true))
            .broadcasted(true)
            .build()
            .await;
        let failed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .confirmed(Some(true))
            .broadcasted(false)
            .build()
            .await;
        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .confirmed(None)
            .broadcasted(true)
            .build()
            .await;
        let value_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .confirmed(Some(true))
            .broadcasted(true)
            .build()
            .await;

        account_handle
            .write()
            .await
            .save_messages(vec![
                received_message.clone(),
                sent_message.clone(),
                failed_message.clone(),
                unconfirmed_message.clone(),
                value_message.clone(),
            ])
            .await
            .unwrap();

        let cases = vec![
            (MessageType::Failed, &failed_message),
            (MessageType::Received, &received_message),
            (MessageType::Sent, &sent_message),
            (MessageType::Unconfirmed, &unconfirmed_message),
            (MessageType::Value, &received_message),
            (MessageType::Confirmed, &received_message),
        ];
        for (tx_type, expected) in cases {
            let messages = account_handle.list_messages(0, 0, Some(tx_type.clone())).await.unwrap();
            assert_eq!(
                messages.len(),
                match tx_type {
                    MessageType::Received => 4,
                    MessageType::Confirmed => 4,
                    MessageType::Value => 5,
                    _ => 1,
                }
            );
            assert_eq!(messages.first().unwrap(), expected);
        }
    }

    #[tokio::test]
    async fn get_message_by_id() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

        let m1 = crate::test_utils::GenerateMessageBuilder::default().build().await;
        let m2 = crate::test_utils::GenerateMessageBuilder::default().build().await;
        account_handle
            .write()
            .await
            .save_messages(vec![m1, m2.clone()])
            .await
            .unwrap();
        assert_eq!(account_handle.read().await.get_message(m2.id()).await.unwrap(), m2);
    }

    #[tokio::test]
    #[ignore]
    async fn list_addresses() {
        crate::test_utils::with_account_manager(
            crate::test_utils::TestType::SigningAndStorage,
            |manager, signer_type| async move {
                let account_handle = crate::test_utils::AccountCreator::new(&manager)
                    .signer_type(signer_type)
                    .create()
                    .await;

                let spent_address = account_handle.latest_address().await;
                let unspent_address1 = account_handle.generate_address().await.unwrap();
                let unspent_address2 = account_handle.generate_address().await.unwrap();

                let spent_tx = crate::test_utils::GenerateMessageBuilder::default()
                    .address(spent_address.clone())
                    .input_address(Some(unspent_address1.address().clone()))
                    .build()
                    .await;

                account_handle
                    .write()
                    .await
                    .save_messages(vec![spent_tx])
                    .await
                    .unwrap();

                assert_eq!(
                    account_handle.read().await.list_unspent_addresses().await.unwrap(),
                    vec![&unspent_address1, &unspent_address2]
                );

                assert_eq!(
                    account_handle.read().await.list_spent_addresses().await.unwrap(),
                    vec![&spent_address]
                );

                assert_eq!(
                    account_handle.read().await.addresses(),
                    &vec![spent_address, unspent_address1, unspent_address2]
                );
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_get_info() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager)
            .addresses(vec![crate::test_utils::generate_random_address()])
            .create()
            .await;

        let node_info = account_handle.get_node_info(None, None, None).await.unwrap();
        println!("{:#?}", node_info);
    }
}

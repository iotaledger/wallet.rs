// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_manager::AccountStore,
    address::{Address, AddressBuilder, AddressWrapper},
    client::ClientOptions,
    message::{Message, MessageType},
    signing::{GenerateAddressMetadata, SignerType},
};

use chrono::prelude::{DateTime, Local};
use getset::{Getters, Setters};
use iota::message::prelude::MessageId;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

mod sync;
pub(crate) use sync::{repost_message, AccountSynchronizeStep, RepostAction};
pub use sync::{AccountSynchronizer, SyncedAccount};

const ACCOUNT_ID_PREFIX: &str = "wallet-account://";

/// The account identifier.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
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
    alias: Option<String>,
    created_at: Option<DateTime<Local>>,
    messages: Vec<Message>,
    addresses: Vec<Address>,
    client_options: ClientOptions,
    signer_type: Option<SignerType>,
    skip_persistance: bool,
}

impl AccountInitialiser {
    /// Initialises the account builder.
    pub(crate) fn new(client_options: ClientOptions, accounts: AccountStore, storage_path: PathBuf) -> Self {
        Self {
            accounts,
            storage_path,
            alias: None,
            created_at: None,
            messages: vec![],
            addresses: vec![],
            client_options,
            #[cfg(feature = "stronghold")]
            signer_type: Some(SignerType::Stronghold),
            #[cfg(not(feature = "stronghold"))]
            signer_type: None,
            skip_persistance: false,
        }
    }

    /// Sets the account type.
    pub fn signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type.replace(signer_type);
        self
    }

    /// Defines the account alias. If not defined, we'll generate one.
    pub fn alias(mut self, alias: impl AsRef<str>) -> Self {
        self.alias = Some(alias.as_ref().to_string());
        self
    }

    /// Time of account creation.
    pub fn created_at(mut self, created_at: DateTime<Local>) -> Self {
        self.created_at = Some(created_at);
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
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    /// Initialises the account.
    pub async fn initialise(self) -> crate::Result<AccountHandle> {
        let accounts = self.accounts.read().await;

        let alias = self.alias.unwrap_or_else(|| format!("Account {}", accounts.len()));
        let signer_type = self.signer_type.ok_or(crate::Error::AccountInitialiseRequiredField(
            crate::error::AccountInitialiseRequiredField::SignerType,
        ))?;
        let created_at = self.created_at.unwrap_or_else(Local::now);

        for account_handle in accounts.values() {
            let account = account_handle.read().await;
            if account.alias() == &alias {
                return Err(crate::Error::AccountAliasAlreadyExists);
            }
        }

        if let Some(latest_account_handle) = accounts.values().last() {
            let latest_account = latest_account_handle.read().await;
            if latest_account.messages().is_empty() && latest_account.balance().total == 0 {
                return Err(crate::Error::LatestAccountIsEmpty);
            }
        }

        let mut account_index = 0;
        for account in accounts.values() {
            if account.read().await.signer_type() == &signer_type {
                account_index += 1;
            }
        }

        let mut account = Account {
            id: account_index.to_string(),
            signer_type: signer_type.clone(),
            index: account_index,
            alias,
            created_at,
            last_synced_at: None,
            messages: self.messages,
            addresses: self.addresses,
            client_options: self.client_options,
            storage_path: self.storage_path,
            skip_persistance: self.skip_persistance,
        };

        let bech32_hrp = crate::client::get_client(account.client_options())
            .read()
            .await
            .get_network_info()
            .bech32_hrp;

        let address = crate::address::get_iota_address(
            &account,
            0,
            false,
            bech32_hrp,
            GenerateAddressMetadata { syncing: false },
        )
        .await?;

        account.addresses.push(
            AddressBuilder::new()
                .address(address.clone())
                .key_index(0)
                .internal(false)
                .outputs(Vec::new())
                .balance(0)
                .build()
                .unwrap(), // safe to unwrap since we provide all required fields
        );

        let mut digest = [0; 32];
        let raw = match address.as_ref() {
            iota::Address::Ed25519(a) => a.as_ref().to_vec(),
            _ => unimplemented!(),
        };
        crypto::hashes::sha::SHA256(&raw, &mut digest);
        account.set_id(format!("{}{}", ACCOUNT_ID_PREFIX, hex::encode(digest)));

        let guard = if self.skip_persistance {
            account.into()
        } else {
            account.save().await?;
            let account_id = account.id().clone();
            let guard: AccountHandle = account.into();
            drop(accounts);
            self.accounts.write().await.insert(account_id, guard.clone());
            guard
        };

        Ok(guard)
    }
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    #[getset(set = "pub(crate)")]
    id: String,
    /// The account's signer type.
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
    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    #[getset(set = "pub")]
    messages: Vec<Message>,
    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    #[getset(set = "pub")]
    addresses: Vec<Address>,
    /// The client options.
    #[serde(rename = "clientOptions")]
    client_options: ClientOptions,
    #[getset(set = "pub(crate)", get = "pub(crate)")]
    storage_path: PathBuf,
    #[getset(set = "pub(crate)", get = "pub(crate)")]
    #[serde(skip)]
    skip_persistance: bool,
}

/// A thread guard over an account.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    inner: Arc<RwLock<Account>>,
    locked_addresses: Arc<Mutex<Vec<AddressWrapper>>>,
}

impl AccountHandle {
    pub(crate) fn locked_addresses(&self) -> Arc<Mutex<Vec<AddressWrapper>>> {
        self.locked_addresses.clone()
    }
}

impl From<Account> for AccountHandle {
    fn from(account: Account) -> Self {
        Self {
            inner: Arc::new(RwLock::new(account)),
            locked_addresses: Default::default(),
        }
    }
}

impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        &self.inner.deref()
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
    #[doc = "Bridge to [Account#messages](struct.Account.html#method.messages).
    This method clones the messages so prefer the using the `read` method to access the account instance."] => messages => Vec<Message>,
    #[doc = "Bridge to [Account#addresses](struct.Account.html#method.addresses).
    This method clones the addresses so prefer the using the `read` method to access the account instance."] => addresses => Vec<Address>,
    #[doc = "Bridge to [Account#client_options](struct.Account.html#method.client_options)."] => client_options => ClientOptions
);

impl AccountHandle {
    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub async fn sync(&self) -> AccountSynchronizer {
        AccountSynchronizer::new(self.clone()).await
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
        let address = crate::address::get_new_address(&account, GenerateAddressMetadata { syncing: false }).await?;

        account
            .do_mut(|account| {
                account.addresses.push(address.clone());
                Ok(())
            })
            .await?;

        let _ = crate::monitor::monitor_address_balance(self.clone(), address.address());

        Ok(address)
    }

    /// Synchronizes the account addresses with the Tangle and returns the latest address in the account,
    /// which is an address without balance.
    pub async fn get_unused_address(&self) -> crate::Result<Address> {
        self.sync()
            .await
            .steps(vec![AccountSynchronizeStep::SyncAddresses])
            .execute()
            .await?;
        // safe to clone since the `sync` guarantees a latest unused address
        Ok(self.latest_address().await)
    }

    /// Syncs the latest address with the Tangle and determines whether it's unused or not.
    /// An unused address is an address without balance and associated message history.
    /// Note that such address might have been used in the past, because the message history might have been pruned by
    /// the node.
    pub async fn is_latest_address_unused(&self) -> crate::Result<bool> {
        let mut latest_address = self.latest_address().await;
        let bech32_hrp = latest_address.address().bech32_hrp().to_string();
        sync::sync_address(&*self.inner.read().await, &mut latest_address, bech32_hrp).await?;
        Ok(*latest_address.balance() == 0 && latest_address.outputs().is_empty())
    }

    /// Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
    pub async fn latest_address(&self) -> Address {
        self.inner.read().await.latest_address().clone()
    }

    /// Bridge to [Account#balance](struct.Account.html#method.balance).
    pub async fn balance(&self) -> AccountBalance {
        self.inner.read().await.balance()
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
    pub async fn list_messages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Vec<Message> {
        self.inner
            .read()
            .await
            .list_messages(count, from, message_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_spent_addresses(&self) -> Vec<Address> {
        self.inner
            .read()
            .await
            .list_spent_addresses()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_unspent_addresses(&self) -> Vec<Address> {
        self.inner
            .read()
            .await
            .list_unspent_addresses()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#get_message](struct.Account.html#method.get_message).
    pub async fn get_message(&self, message_id: &MessageId) -> Option<Message> {
        self.inner.read().await.get_message(message_id).cloned()
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
        if !self.skip_persistance {
            let storage_path = self.storage_path.clone();
            crate::storage::get(&storage_path)
                .await?
                .lock()
                .await
                .set(&self.id, serde_json::to_string(&self)?)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn do_mut<R>(&mut self, f: impl FnOnce(&mut Self) -> crate::Result<R>) -> crate::Result<R> {
        let res = f(self)?;
        self.save().await?;
        Ok(res)
    }

    /// Returns the most recent address of the account.
    pub fn latest_address(&self) -> &Address {
        // the addresses list is never empty because we generate an address on the accout creation
        self.addresses
            .iter()
            .filter(|a| !a.internal())
            .max_by_key(|a| a.key_index())
            .unwrap()
    }

    /// Gets the account balance information.
    pub fn balance(&self) -> AccountBalance {
        let (incoming, outgoing) =
            self.list_messages(0, 0, None)
                .iter()
                .fold((0, 0), |(incoming, outgoing), message| {
                    if *message.incoming() {
                        (incoming + *message.value(), outgoing)
                    } else {
                        (incoming, outgoing + *message.value())
                    }
                });
        AccountBalance {
            total: self.addresses.iter().fold(0, |acc, address| acc + address.balance()),
            available: self
                .addresses()
                .iter()
                .fold(0, |acc, addr| acc + addr.available_balance(&self)),
            incoming,
            outgoing,
        }
    }

    /// Updates the account alias.
    pub async fn set_alias(&mut self, alias: impl AsRef<str>) -> crate::Result<()> {
        let alias = alias.as_ref().to_string();

        self.alias = alias;
        self.save().await
    }

    /// Updates the account's client options.
    pub async fn set_client_options(&mut self, options: ClientOptions) -> crate::Result<()> {
        self.client_options = options;

        let bech32_hrp = crate::client::get_client(&self.client_options)
            .read()
            .await
            .get_network_info()
            .bech32_hrp;
        for address in &mut self.addresses {
            address.set_bech32_hrp(bech32_hrp.to_string());
        }

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
    /// ```
    /// use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder, message::MessageType, signing::SignerType};
    /// # use rand::{distributions::Alphanumeric, thread_rng, Rng};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     # let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    ///     # let storage_path = std::path::PathBuf::from(format!("./test-storage/{}", storage_path));
    ///     // gets 10 received messages, skipping the first 5 most recent messages.
    ///     let client_options = ClientOptionsBuilder::node("https://api.lb-0.testnet.chrysalis2.com")
    ///         .expect("invalid node URL")
    ///         .build();
    ///     let mut manager = AccountManager::builder().with_storage("./test-storage", ManagerStorage::Stronghold, None).unwrap().finish().await.unwrap();
    ///     # use iota_wallet::account_manager::ManagerStorage;
    ///     # #[cfg(all(feature = "stronghold-storage", feature = "sqlite-storage"))]
    ///     # let default_storage = ManagerStorage::Stronghold;
    ///     # #[cfg(all(feature = "stronghold-storage", not(feature = "sqlite-storage")))]
    ///     # let default_storage = ManagerStorage::Stronghold;
    ///     # #[cfg(all(feature = "sqlite-storage", not(feature = "stronghold-storage")))]
    ///     # let default_storage = ManagerStorage::Sqlite;
    ///     # let mut manager = AccountManager::builder().with_storage(storage_path, default_storage, None).unwrap().finish().await.unwrap();
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
    pub fn list_messages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Vec<&Message> {
        let mut messages: Vec<&Message> = vec![];
        for message in self.messages.iter() {
            // if we already found a message with the same payload,
            // this is a reattachment message
            if let Some(original_message_index) = messages.iter().position(|m| m.payload() == message.payload()) {
                let original_message = messages[original_message_index];
                // if the original message was confirmed, we ignore this reattachment
                if original_message.confirmed().unwrap_or(false) {
                    continue;
                } else {
                    // remove the original message otherwise
                    messages.remove(original_message_index);
                }
            }
            let should_push = if let Some(message_type) = message_type.clone() {
                match message_type {
                    MessageType::Received => *message.incoming(),
                    MessageType::Sent => !message.incoming(),
                    MessageType::Failed => !message.broadcasted(),
                    MessageType::Unconfirmed => !message.confirmed().unwrap_or(false),
                    MessageType::Value => *message.value() > 0,
                }
            } else {
                true
            };
            if should_push {
                messages.push(message);
            }
        }
        let messages_iter = messages.into_iter().skip(from);
        if count == 0 {
            messages_iter.collect()
        } else {
            messages_iter.take(count).collect()
        }
    }

    /// Gets the spent addresses.
    pub fn list_spent_addresses(&self) -> Vec<&Address> {
        self.addresses
            .iter()
            .filter(|address| !crate::address::is_unspent(&self, address.address()))
            .collect()
    }

    /// Gets the spent addresses.
    pub fn list_unspent_addresses(&self) -> Vec<&Address> {
        self.addresses
            .iter()
            .filter(|address| crate::address::is_unspent(&self, address.address()))
            .collect()
    }

    #[doc(hidden)]
    pub fn append_messages(&mut self, messages: Vec<Message>) {
        self.messages.extend(messages);
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

    pub(crate) fn addresses_mut(&mut self) -> &mut Vec<Address> {
        &mut self.addresses
    }

    pub(crate) fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    /// Gets a message with the given id associated with this account.
    pub fn get_message(&self, message_id: &MessageId) -> Option<&Message> {
        self.messages.iter().find(|tx| tx.id() == message_id)
    }
}

#[cfg(test)]
mod tests {
    use super::AccountHandle;
    use crate::{
        account_manager::AccountManager,
        address::{Address, AddressBuilder, AddressOutput},
        client::ClientOptionsBuilder,
        message::{Message, MessageType},
    };
    use iota::{MessageId, TransactionId};

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

            let updated_client_options =
                ClientOptionsBuilder::nodes(&["http://test.wallet", "http://test.wallet/set-client-options"])
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

        assert_bridge_method!(
            id,
            signer_type,
            index,
            alias,
            created_at,
            messages,
            addresses,
            client_options
        );
    }

    fn _generate_address_output(value: u64) -> AddressOutput {
        let mut tx_id = [0; 32];
        crypto::rand::fill(&mut tx_id).unwrap();
        AddressOutput {
            transaction_id: TransactionId::new(tx_id),
            message_id: MessageId::new([0; 32]),
            index: 0,
            amount: value,
            is_spent: false,
            address: crate::test_utils::generate_random_iota_address(),
        }
    }

    async fn _generate_account(manager: &AccountManager, messages: Vec<Message>) -> (AccountHandle, Address, u64) {
        let balance = 30;
        let first_address = AddressBuilder::new()
            .address(crate::test_utils::generate_random_iota_address())
            .key_index(0)
            .balance(balance / 2_u64)
            .outputs(vec![_generate_address_output(balance / 2_u64)])
            .build()
            .unwrap();
        let second_address = AddressBuilder::new()
            .address(crate::test_utils::generate_random_iota_address())
            .key_index(1)
            .balance(balance / 2_u64)
            .outputs(vec![_generate_address_output(balance / 2_u64)])
            .build()
            .unwrap();

        let addresses = vec![second_address.clone(), first_address];
        let account_handle = crate::test_utils::AccountCreator::new(&manager)
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

                let account_next_address = crate::address::get_new_address(
                    &*account_handle.read().await,
                    crate::signing::GenerateAddressMetadata { syncing: false },
                )
                .await
                .unwrap();
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
        assert_eq!(account_handle.read().await.balance().total, balance);
    }

    #[tokio::test]
    async fn available_balance() {
        let manager = crate::test_utils::get_account_manager().await;
        let (account_handle, _, balance) = _generate_account(&manager, vec![]).await;
        assert_eq!(account_handle.read().await.balance().available, balance);

        let first_address = {
            let mut account = account_handle.write().await;
            let address = account.addresses_mut().iter_mut().next().unwrap();
            address.outputs = vec![_generate_address_output(15)];
            address.clone()
        };
        let second_address = {
            let mut account = account_handle.write().await;
            let addresses = account.addresses_mut();
            let mut iter = addresses.iter_mut();
            iter.next();
            let address = iter.next().unwrap();
            address.outputs = vec![_generate_address_output(15)];
            address.clone()
        };

        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(first_address.clone())
            .value(15)
            .input_transaction_id(first_address.outputs[0].transaction_id)
            .build();
        let confirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(second_address.clone())
            .value(10)
            .input_transaction_id(second_address.outputs[0].transaction_id)
            .confirmed(true)
            .build();

        account_handle
            .write()
            .await
            .append_messages(vec![unconfirmed_message.clone(), confirmed_message]);

        assert_eq!(
            account_handle.read().await.balance().available,
            balance - *unconfirmed_message.value()
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
            .incoming(true)
            .build();
        let failed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .broadcasted(false)
            .build();
        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .confirmed(false)
            .build();
        let value_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .build();
        account_handle.write().await.append_messages(vec![
            received_message,
            failed_message,
            unconfirmed_message,
            value_message,
        ]);

        let txs = account_handle.list_messages(4, 0, None).await;
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
            .incoming(true)
            .confirmed(true)
            .broadcasted(true)
            .build();
        let sent_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(external_address.clone())
            .incoming(false)
            .confirmed(true)
            .broadcasted(true)
            .build();
        let failed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .incoming(false)
            .confirmed(true)
            .broadcasted(false)
            .build();
        let unconfirmed_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .incoming(false)
            .confirmed(false)
            .broadcasted(true)
            .build();
        let value_message = crate::test_utils::GenerateMessageBuilder::default()
            .address(latest_address.clone())
            .incoming(false)
            .confirmed(true)
            .broadcasted(true)
            .build();

        account_handle.write().await.append_messages(vec![
            received_message.clone(),
            sent_message.clone(),
            failed_message.clone(),
            unconfirmed_message.clone(),
            value_message.clone(),
        ]);

        let cases = vec![
            (MessageType::Failed, &failed_message),
            (MessageType::Received, &received_message),
            (MessageType::Sent, &sent_message),
            (MessageType::Unconfirmed, &unconfirmed_message),
            (MessageType::Value, &value_message),
        ];
        for (tx_type, expected) in cases {
            let failed_messages = account_handle.list_messages(0, 0, Some(tx_type.clone())).await;
            assert_eq!(
                failed_messages.len(),
                match tx_type {
                    MessageType::Sent => 4,
                    MessageType::Value => 5,
                    _ => 1,
                }
            );
            assert_eq!(failed_messages.first().unwrap(), expected);
        }
    }

    #[tokio::test]
    async fn get_message_by_id() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;

        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        account_handle.write().await.append_messages(vec![
            crate::test_utils::GenerateMessageBuilder::default().build(),
            message.clone(),
        ]);
        assert_eq!(account_handle.read().await.get_message(message.id()).unwrap(), &message);
    }

    #[tokio::test]
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
                    .incoming(false)
                    .build();

                account_handle.write().await.append_messages(vec![spent_tx]);

                assert_eq!(
                    account_handle.read().await.list_unspent_addresses(),
                    vec![&unspent_address1, &unspent_address2]
                );

                assert_eq!(account_handle.read().await.list_spent_addresses(), vec![&spent_address]);

                assert_eq!(
                    account_handle.read().await.addresses(),
                    &vec![spent_address, unspent_address1, unspent_address2]
                );
            },
        )
        .await;
    }
}

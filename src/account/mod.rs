// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_manager::AccountStore,
    address::{Address, AddressWrapper},
    client::ClientOptions,
    message::{Message, MessageType},
    signing::{GenerateAddressMetadata, SignerType},
};

use chrono::prelude::{DateTime, Local, Utc};
use getset::{Getters, Setters};
use iota::message::prelude::MessageId;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

mod sync;
pub(crate) use sync::{repost_message, RepostAction};
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

        if let Some(latest_account_handle) = accounts.values().last() {
            let latest_account = latest_account_handle.read().await;
            if latest_account.messages().is_empty() && latest_account.total_balance() == 0 {
                return Err(crate::Error::LatestAccountIsEmpty);
            }
        }

        let mut account = Account {
            id: accounts.len().to_string(),
            signer_type: signer_type.clone(),
            index: accounts.len(),
            alias,
            created_at,
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
    #[doc = "Bridge to [Account#messages](struct.Account.html#method.messages).
    This method clones the addresses so prefer the using the `read` method to access the account instance."] => messages => Vec<Message>,
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

    /// Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
    pub async fn latest_address(&self) -> Option<Address> {
        self.inner.read().await.latest_address().cloned()
    }

    /// Bridge to [Account#total_balance](struct.Account.html#method.total_balance).
    pub async fn total_balance(&self) -> u64 {
        self.inner.read().await.total_balance()
    }

    /// Bridge to [Account#available_balance](struct.Account.html#method.available_balance).
    pub async fn available_balance(&self) -> u64 {
        self.inner.read().await.available_balance()
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
    pub fn latest_address(&self) -> Option<&Address> {
        self.addresses
            .iter()
            .filter(|a| !a.internal())
            .max_by_key(|a| a.key_index())
    }

    /// Gets the account's total balance.
    /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
    pub fn total_balance(&self) -> u64 {
        self.addresses.iter().fold(0, |acc, address| acc + address.balance())
    }

    /// Gets the account's available balance.
    /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
    ///
    /// The available balance is the balance users are allowed to spend.
    /// For example, if a user with 50i total account balance has made a transaction spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub fn available_balance(&self) -> u64 {
        self.addresses()
            .iter()
            .fold(0, |acc, addr| acc + addr.available_balance(&self))
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
    /// * `count` - Number of (most recent) transactions to fetch.
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
    ///     let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
    ///         .expect("invalid node URL")
    ///         .build();
    ///     let mut manager = AccountManager::builder().finish().await.unwrap();
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

/// Data returned from the account initialisation.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct InitialisedAccount<'a> {
    /// The account identifier.
    id: &'a str,
    /// The account alias.
    alias: &'a str,
    /// Seed address history.
    addresses: Vec<Address>,
    /// Seed transaction history.
    transactions: Vec<Message>,
    /// Account creation time.
    created_at: DateTime<Local>,
    /// Time when the account was last synced with the tangle.
    last_synced_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptionsBuilder;

    #[tokio::test]
    async fn set_alias() {
        let manager = crate::test_utils::get_account_manager().await;

        let updated_alias = "updated alias";
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let account_id = {
            let account_handle = manager
                .create_account(client_options)
                .unwrap()
                .alias("alias")
                .initialise()
                .await
                .expect("failed to add account");

            account_handle.set_alias(updated_alias).await.unwrap();
            account_handle.id().await
        };

        let account_in_storage = manager
            .get_account(&account_id)
            .await
            .expect("failed to get account from storage");
        let account_in_storage = account_in_storage.read().await;
        assert_eq!(account_in_storage.alias().to_string(), updated_alias.to_string());
    }
}

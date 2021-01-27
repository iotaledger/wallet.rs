// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::address::Address;
use crate::client::ClientOptions;
use crate::message::{Message, MessageType};

use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota::message::prelude::MessageId;
use serde::{Deserialize, Serialize};

use std::convert::TryInto;
use std::path::PathBuf;

mod sync;
pub(crate) use sync::{repost_message, RepostAction};
pub use sync::{AccountSynchronizer, SyncedAccount, TransferMetadata};

/// The account identifier.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum AccountIdentifier {
    /// A hex string of the stronghold record id identifier.
    Id(String),
    /// An index identifier.
    Index(u64),
}

// When the identifier is a stronghold id.
impl From<&String> for AccountIdentifier {
    fn from(value: &String) -> Self {
        Self::Id(value.clone())
    }
}

impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::Id(value)
    }
}

// When the identifier is an id.
impl From<u64> for AccountIdentifier {
    fn from(value: u64) -> Self {
        Self::Index(value)
    }
}

/// Account initialiser.
pub struct AccountInitialiser<'a> {
    mnemonic: Option<String>,
    alias: Option<String>,
    created_at: Option<DateTime<Utc>>,
    messages: Vec<Message>,
    addresses: Vec<Address>,
    client_options: ClientOptions,
    skip_persistance: bool,
    storage_path: &'a PathBuf,
}

impl<'a> AccountInitialiser<'a> {
    /// Initialises the account builder.
    pub(crate) fn new(client_options: ClientOptions, storage_path: &'a PathBuf) -> Self {
        Self {
            mnemonic: None,
            alias: None,
            created_at: None,
            messages: vec![],
            addresses: vec![],
            client_options,
            skip_persistance: false,
            storage_path,
        }
    }

    /// Defines the account BIP-39 mnemonic.
    /// When importing an account from stronghold, the mnemonic won't be required.
    pub fn mnemonic(mut self, mnemonic: impl AsRef<str>) -> Self {
        self.mnemonic = Some(mnemonic.as_ref().to_string());
        self
    }

    /// Defines the account alias. If not defined, we'll generate one.
    pub fn alias(mut self, alias: impl AsRef<str>) -> Self {
        self.alias = Some(alias.as_ref().to_string());
        self
    }

    /// Time of account creation.
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
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

    pub(crate) fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    /// Initialises the account.
    pub fn initialise(self) -> crate::Result<Account> {
        let accounts =
            crate::storage::with_adapter(self.storage_path, |storage| storage.get_all())?;
        let alias = self
            .alias
            .unwrap_or_else(|| format!("Account {}", accounts.len()));
        let created_at = self.created_at.unwrap_or_else(chrono::Utc::now);
        let created_at_timestamp: u128 = created_at.timestamp().try_into().unwrap(); // safe to unwrap since it's > 0
        let mnemonic = self.mnemonic;

        // check for empty latest account only when not skipping persistance (account discovery process)
        if !self.skip_persistance {
            if let Some(latest_account) = accounts.last() {
                let latest_account: Account = serde_json::from_str(&latest_account)?;
                if latest_account.messages().is_empty() && latest_account.total_balance() == 0 {
                    return Err(crate::WalletError::LatestAccountIsEmpty);
                }
            }
        }

        let stronghold_account_res: crate::Result<stronghold::Account> =
            crate::with_stronghold_from_path(&self.storage_path, |stronghold| {
                let account = match mnemonic {
                    Some(mnemonic) => stronghold.account_import(
                        Some(created_at_timestamp),
                        Some(created_at_timestamp),
                        mnemonic,
                        Some("password"),
                    )?,
                    None => stronghold.account_create(Some("password".to_string()))?,
                };
                Ok(account)
            });
        let stronghold_account = stronghold_account_res?;

        let id = stronghold_account.id();
        let id = hex::encode(id);
        let account_id: AccountIdentifier = id.clone().into();

        let account = Account {
            id,
            index: accounts.len(),
            alias,
            created_at,
            messages: self.messages,
            addresses: self.addresses,
            client_options: self.client_options,
            storage_path: self.storage_path.clone(),
            has_pending_changes: false,
        };
        if !self.skip_persistance {
            crate::storage::with_adapter(&self.storage_path, |storage| {
                storage.set(account_id, serde_json::to_string(&account)?)
            })?;
        }
        Ok(account)
    }
}

pub(crate) fn account_id_to_stronghold_record_id(account_id: &str) -> crate::Result<[u8; 32]> {
    let decoded =
        hex::decode(account_id).map_err(|_| anyhow::anyhow!("account id must be a hex string"))?;
    let id: [u8; 32] = decoded
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid account id length"))?;
    Ok(id)
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    id: String,
    /// The account index
    index: usize,
    /// The account alias.
    alias: String,
    /// Time of account creation.
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
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
    #[doc(hidden)]
    #[serde(skip)]
    has_pending_changes: bool,
}

impl Account {
    /// Returns the most recent address of the account.
    pub fn latest_address(&self) -> Option<&Address> {
        self.addresses
            .iter()
            .filter(|a| !a.internal())
            .max_by_key(|a| a.key_index())
    }

    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub fn sync(&'_ mut self) -> AccountSynchronizer<'_> {
        AccountSynchronizer::new(self, self.storage_path.clone())
    }

    /// Gets the account's total balance.
    /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
    pub fn total_balance(&self) -> u64 {
        self.addresses
            .iter()
            .fold(0, |acc, address| acc + address.balance())
    }

    /// Gets the account's available balance.
    /// It's read directly from the storage. To read the latest account balance, you should `sync` first.
    ///
    /// The available balance is the balance users are allowed to spend.
    /// For example, if a user with 50i total account balance has made a transaction spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub fn available_balance(&self) -> u64 {
        let total_balance = self.total_balance();
        let spent = self
            .list_messages(0, 0, Some(MessageType::Sent))
            .iter()
            .fold(0, |acc, message| {
                let val = if *message.confirmed() {
                    0
                } else {
                    *message.value()
                };
                acc + val
            });
        total_balance - (spent as u64)
    }

    /// Updates the account alias.
    pub fn set_alias(&mut self, alias: impl AsRef<str>) {
        let alias = alias.as_ref().to_string();
        if !self.has_pending_changes {
            self.has_pending_changes = alias != self.alias;
        }

        self.alias = alias;
    }

    /// Updates the account's client options.
    pub fn set_client_options(&mut self, options: ClientOptions) {
        if !self.has_pending_changes {
            self.has_pending_changes = options != self.client_options;
        }
        self.client_options = options;
    }

    /// Saves the pending changes on the account.
    /// This is automatically performed when the account goes out of scope.
    pub fn save_pending_changes(&mut self) -> crate::Result<()> {
        if self.has_pending_changes {
            crate::storage::with_adapter(&self.storage_path, |storage| {
                storage.set(self.id.clone().into(), serde_json::to_string(&self)?)
            })?;
            self.has_pending_changes = false;
        }
        Ok(())
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
    /// use iota_wallet::message::MessageType;
    /// use iota_wallet::account_manager::AccountManager;
    /// use iota_wallet::client::ClientOptionsBuilder;
    /// # use rand::{thread_rng, Rng};
    ///
    /// # let storage_path: String = thread_rng().gen_ascii_chars().take(10).collect();
    /// # let storage_path = std::path::PathBuf::from(format!("./example-database/{}", storage_path));
    /// // gets 10 received messages, skipping the first 5 most recent messages.
    /// let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
    ///  .expect("invalid node URL")
    ///  .build();
    /// let mut manager = AccountManager::new().unwrap();
    /// # let mut manager = AccountManager::with_storage_path(storage_path).unwrap();
    /// manager.set_stronghold_password("password").unwrap();
    /// let mut account = manager.create_account(client_options)
    ///   .initialise()
    ///   .expect("failed to add account");
    /// account.list_messages(10, 5, Some(MessageType::Received));
    /// ```
    pub fn list_messages(
        &self,
        count: usize,
        from: usize,
        message_type: Option<MessageType>,
    ) -> Vec<&Message> {
        let mut messages: Vec<&Message> = vec![];
        for message in self.messages.iter() {
            // if we already found a message with the same payload,
            // this is a reattachment message
            if let Some(original_message_index) = messages
                .iter()
                .position(|m| m.payload() == message.payload())
            {
                let original_message = messages[original_message_index];
                // if the original message was confirmed, we ignore this reattachment
                if *original_message.confirmed() {
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
                    MessageType::Unconfirmed => !message.confirmed(),
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

    /// Gets the addresses linked to this account.
    ///
    /// * `unspent` - Whether it should get only unspent addresses or not.
    pub fn list_addresses(&self, unspent: bool) -> Vec<&Address> {
        self.addresses
            .iter()
            .filter(|address| crate::address::is_unspent(&self, address.address()) == unspent)
            .collect()
    }

    /// Gets a new unused address and links it to this account.
    pub fn generate_address(&mut self) -> crate::Result<Address> {
        let address = crate::address::get_new_address(&self)?;
        self.addresses.push(address.clone());

        let id: AccountIdentifier = self.id.clone().into();
        crate::storage::with_adapter(&self.storage_path, |storage| {
            storage.set(id, serde_json::to_string(self)?)
        })?;

        // ignore errors because we fallback to the polling system
        let _ = crate::monitor::monitor_address_balance(&self, address.address());
        Ok(address)
    }

    #[doc(hidden)]
    pub fn append_messages(&mut self, messages: Vec<Message>) {
        self.messages.extend(messages.iter().cloned());
    }

    pub(crate) fn append_addresses(&mut self, addresses: Vec<Address>) {
        addresses.into_iter().for_each(|address| {
            match self.addresses.iter().position(|a| a == &address) {
                Some(index) => {
                    self.addresses[index] = address;
                }
                None => {
                    self.addresses.push(address);
                }
            }
        });
    }

    pub(crate) fn addresses_mut(&mut self) -> &mut Vec<Address> {
        &mut self.addresses
    }

    /// Gets a message with the given id associated with this account.
    pub fn get_message(&self, message_id: &MessageId) -> Option<&Message> {
        self.messages.iter().find(|tx| tx.id() == message_id)
    }
}

impl Drop for Account {
    fn drop(&mut self) {
        let _ = self.save_pending_changes();
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
    created_at: DateTime<Utc>,
    /// Time when the account was last synced with the tangle.
    last_synced_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptionsBuilder;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn set_alias() {
            let manager = crate::test_utils::get_account_manager();

            let updated_alias = "updated alias";
            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account_id = {
                let mut account = manager
                .create_account(client_options)
                .alias("alias")
                .initialise()
                .expect("failed to add account");

                account.set_alias(updated_alias);
                account.id().into()
            };

            let account_in_storage = manager
                .get_account(account_id)
                .expect("failed to get account from storage");
            assert_eq!(
                account_in_storage.alias().to_string(),
                updated_alias.to_string()
            );
        }
    }
}

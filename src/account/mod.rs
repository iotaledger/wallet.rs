use crate::address::Address;
use crate::client::ClientOptions;
use crate::message::{Message, MessageType};

use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota::transaction::prelude::MessageId;
use serde::{Deserialize, Serialize};

use std::convert::TryInto;

mod sync;
pub use sync::{AccountSynchronizer, SyncedAccount};

/// The account identifier.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum AccountIdentifier {
    /// A stronghold record id identifier.
    Id([u8; 32]),
    /// An index identifier.
    Index(u64),
}

// When the identifier is a stronghold id.
impl From<[u8; 32]> for AccountIdentifier {
    fn from(value: [u8; 32]) -> Self {
        Self::Id(value)
    }
}
impl From<&[u8; 32]> for AccountIdentifier {
    fn from(value: &[u8; 32]) -> Self {
        Self::Id(*value)
    }
}

// When the identifier is an id.
impl From<u64> for AccountIdentifier {
    fn from(value: u64) -> Self {
        Self::Index(value)
    }
}

/// Account initialiser.
pub struct AccountInitialiser {
    mnemonic: Option<String>,
    alias: Option<String>,
    created_at: Option<DateTime<Utc>>,
    messages: Vec<Message>,
    addresses: Vec<Address>,
    client_options: ClientOptions,
}

impl AccountInitialiser {
    /// Initialises the account builder.
    pub(crate) fn new(client_options: ClientOptions) -> Self {
        Self {
            mnemonic: None,
            alias: None,
            created_at: None,
            messages: vec![],
            addresses: vec![],
            client_options,
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

    /// Initialises the account.
    pub fn initialise(self) -> crate::Result<Account> {
        let alias = self.alias.unwrap_or_else(|| "".to_string());
        let created_at = self.created_at.unwrap_or_else(chrono::Utc::now);
        let created_at_timestamp: u128 = created_at.timestamp().try_into().unwrap(); // safe to unwrap since it's > 0
        let mnemonic = self.mnemonic;

        let adapter = crate::storage::get_adapter()?;
        let accounts = adapter.get_all()?;

        if let Some(latest_account) = accounts.last() {
            let latest_account: Account = serde_json::from_str(&latest_account)?;
            if latest_account.messages().is_empty() && latest_account.total_balance() == 0 {
                return Err(anyhow::anyhow!("can't create accounts when the latest account doesn't have message history and balance"));
            }
        }

        let stronghold_account_res: crate::Result<stronghold::Account> =
            crate::with_stronghold(|stronghold| {
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
        let account_id: AccountIdentifier = id.clone().into();

        let account = Account {
            id: *id,
            alias,
            created_at,
            messages: self.messages,
            addresses: self.addresses,
            client_options: self.client_options,
        };
        adapter.set(account_id, serde_json::to_string(&account)?)?;
        Ok(account)
    }
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    id: [u8; 32],
    /// The account alias.
    alias: String,
    /// Time of account creation.
    created_at: DateTime<Utc>,
    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    #[getset(set = "pub(crate)")]
    messages: Vec<Message>,
    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    #[getset(set = "pub(crate)")]
    addresses: Vec<Address>,
    /// The client options.
    client_options: ClientOptions,
}

impl Account {
    /// Returns the most recent address of the account.
    pub fn latest_address(&self) -> Option<&Address> {
        self.addresses.iter().max_by_key(|a| a.key_index())
    }
    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub fn sync(&mut self) -> AccountSynchronizer<'_> {
        AccountSynchronizer::new(self)
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
    /// For example, if a user with 50i total account balance has made a message spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub fn available_balance(&self) -> u64 {
        let total_balance = self.total_balance();
        let spent = self.messages.iter().fold(0, |acc, message| {
            let val = if *message.confirmed() {
                0
            } else {
                message.value().without_denomination()
            };
            acc + val
        });
        total_balance - (spent as u64)
    }

    /// Updates the account alias.
    pub fn set_alias(&mut self, alias: impl AsRef<str>) -> crate::Result<()> {
        self.alias = alias.as_ref().to_string();
        crate::storage::get_adapter()?.set(self.id.into(), serde_json::to_string(self)?)?;
        Ok(())
    }

    /// Gets a list of messages on this account.
    /// It's fetched from the storage. To ensure the database is updated with the latest messages,
    /// `sync` should be called first.
    ///
    /// * `count` - Number of (most recent) messages to fetch.
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
    /// # iota_wallet::storage::set_storage_path(&storage_path).unwrap();
    /// // gets 10 received messages, skipping the first 5 most recent messages.
    /// let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
    ///  .expect("invalid node URL")
    ///  .build();
    /// let mut manager = AccountManager::new();
    /// manager.set_stronghold_password("password").unwrap();
    /// let mut account = manager.create_account(client_options)
    ///   .initialise()
    ///   .expect("failed to add account");
    /// account.list_messages(10, 5, Some(MessageType::Received));
    /// ```
    pub fn list_messages(
        &self,
        count: u64,
        from: u64,
        message_type: Option<MessageType>,
    ) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|message| {
                if let Some(message_type) = message_type.clone() {
                    match message_type {
                        MessageType::Received => self.addresses.contains(&message.address()),
                        MessageType::Sent => !self.addresses.contains(&message.address()),
                        MessageType::Failed => !message.broadcasted(),
                        MessageType::Unconfirmed => !message.confirmed(),
                        MessageType::Value => message.value().without_denomination() > 0,
                    }
                } else {
                    true
                }
            })
            .collect()
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
    pub async fn generate_address(&mut self) -> crate::Result<Address> {
        let address = crate::address::get_new_address(&self, false).await?;
        self.addresses.push(address.clone());
        crate::storage::get_adapter()?.set(self.id.into(), serde_json::to_string(self)?)?;
        Ok(address)
    }

    pub(crate) fn append_messages(&mut self, messages: Vec<Message>) {
        self.messages.extend(messages.iter().cloned());
    }

    /// Gets a message with the given id associated with this account.
    pub fn get_message(&self, message_id: &MessageId) -> Option<&Message> {
        self.messages
            .iter()
            .find(|tx| tx.message_id() == message_id)
    }

    /// Gets the account index.
    pub fn index(&self) -> crate::Result<usize> {
        let adapter = crate::storage::get_adapter()?;
        let accounts = adapter.get_all()?;
        let account_json = serde_json::to_string(&self)?;
        let index = accounts
            .iter()
            .position(|acc| acc == &account_json)
            .unwrap();
        Ok(index)
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
    /// Seed message history.
    messages: Vec<Message>,
    /// Account creation time.
    created_at: DateTime<Utc>,
    /// Time when the account was last synced with the tangle.
    last_synced_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::Account;
    use crate::account_manager::AccountManager;
    use crate::address::{Address, AddressBuilder};
    use crate::client::ClientOptionsBuilder;
    use crate::message::{Message, MessageType};

    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        // asserts that the `set_alias` function updates the account alias in storage
        fn set_alias() {
            let manager = crate::test_utils::get_account_manager();
            let updated_alias = "updated alias";
            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let updated_alias = "updated alias";
            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let mut account = manager
                .create_account(client_options)
                .alias("alias")
                .initialise()
                .expect("failed to add account");

            account
                .set_alias(updated_alias)
                .expect("failed to update alias");
            let account_in_storage = manager
                .get_account(account.id().into())
                .expect("failed to get account from storage");
            assert_eq!(
                account_in_storage.alias().to_string(),
                updated_alias.to_string()
            );
        }
    }

    fn _generate_account(
        manager: &AccountManager,
        messages: Vec<Message>,
    ) -> (Account, Address, u64) {
        let address = crate::test_utils::generate_random_iota_address();
        let balance = 30;
        let first_address = AddressBuilder::new()
            .address(address.clone())
            .key_index(0)
            .balance(balance / 2 as u64)
            .build()
            .unwrap();
        let second_address = AddressBuilder::new()
            .address(address)
            .key_index(1)
            .balance(balance / 2 as u64)
            .build()
            .unwrap();

        let addresses = vec![second_address.clone(), first_address];
        let account = crate::test_utils::create_account(&manager, addresses);
        (account, second_address, balance)
    }

    #[test]
    fn latest_address() {
        let manager = crate::test_utils::get_account_manager();
        let (account, latest_address, _) = _generate_account(&manager, vec![]);
        assert_eq!(account.latest_address(), Some(&latest_address));
    }

    #[test]
    fn total_balance() {
        let manager = crate::test_utils::get_account_manager();
        let (account, _, balance) = _generate_account(&manager, vec![]);
        assert_eq!(account.total_balance(), balance);
    }

    #[test]
    fn available_balance() {
        let manager = crate::test_utils::get_account_manager();
        let (account, _, balance) = _generate_account(&manager, vec![]);
        assert_eq!(account.available_balance(), balance);

        let unconfirmed_message = crate::test_utils::generate_message(
            15,
            account.addresses().first().unwrap().clone(),
            false,
            false,
        );
        let confirmed_message = crate::test_utils::generate_message(
            10,
            account.addresses().first().unwrap().clone(),
            true,
            true,
        );
        let (account, _, balance) = _generate_account(
            &manager,
            vec![unconfirmed_message.clone(), confirmed_message],
        );
        assert_eq!(
            account.available_balance(),
            balance - unconfirmed_message.value().without_denomination() as u64
        );
    }

    #[test]
    fn list_all_messages() {
        let manager = crate::test_utils::get_account_manager();
        let (mut account, _, _) = _generate_account(&manager, vec![]);
        let received_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            true,
            true,
        );
        let failed_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            true,
            false,
        );
        let unconfirmed_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            false,
            true,
        );
        let value_message = crate::test_utils::generate_message(
            4,
            account.latest_address().unwrap().clone(),
            true,
            true,
        );
        account.append_messages(vec![
            received_message,
            failed_message,
            unconfirmed_message,
            value_message,
        ]);

        let txs = account.list_messages(4, 0, None);
        assert_eq!(txs.len(), 4);
    }

    #[test]
    fn list_messages_by_type() {
        let manager = crate::test_utils::get_account_manager();
        let (mut account, _, _) = _generate_account(&manager, vec![]);

        let external_address = AddressBuilder::new()
            .address(crate::test_utils::generate_random_iota_address())
            .key_index(0)
            .balance(0)
            .build()
            .unwrap();

        let received_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            true,
            true,
        );
        let sent_message =
            crate::test_utils::generate_message(0, external_address.clone(), true, true);
        let failed_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            true,
            false,
        );
        let unconfirmed_message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            false,
            true,
        );
        let value_message = crate::test_utils::generate_message(
            5,
            account.latest_address().unwrap().clone(),
            true,
            true,
        );
        account.append_messages(vec![
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
            let failed_messages = account.list_messages(1, 0, Some(tx_type.clone()));
            assert_eq!(
                failed_messages.len(),
                if tx_type == MessageType::Received {
                    4
                } else {
                    1
                }
            );
            assert_eq!(failed_messages.first().unwrap(), &expected);
        }
    }

    #[test]
    fn get_message_by_id() {
        let manager = crate::test_utils::get_account_manager();
        let (mut account, _, _) = _generate_account(&manager, vec![]);

        let message = crate::test_utils::generate_message(
            0,
            account.latest_address().unwrap().clone(),
            true,
            true,
        );
        account.append_messages(vec![
            crate::test_utils::generate_message(
                10,
                account.latest_address().unwrap().clone(),
                true,
                true,
            ),
            message.clone(),
        ]);
        assert_eq!(account.get_message(message.message_id()).unwrap(), &message);
    }

    // TODO list_addresses, generate_addresses tests
}

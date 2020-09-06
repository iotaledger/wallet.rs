use crate::address::Address;
use crate::client::ClientOptions;
use crate::transaction::{Transaction, TransactionType};

use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota::crypto::ternary::Hash;
use serde::{Deserialize, Serialize};

use std::convert::TryInto;

mod sync;
pub use sync::{AccountSynchronizer, SyncedAccount};

/// The account identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountIdentifier {
    /// An Id (string) identifier.
    Id(String),
    /// An index identifier.
    Index(u64),
}

// When the identifier is a String (id).
impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::Id(value)
    }
}

// When the identifier is an index.
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
    transactions: Vec<Transaction>,
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
            transactions: vec![],
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

    /// Transactions associated with the seed.
    /// The account can be initialised with locally stored transactions.
    pub fn transactions(mut self, transactions: Vec<Transaction>) -> Self {
        self.transactions = transactions;
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

        let stronghold_account = crate::with_stronghold(|stronghold| match mnemonic {
            Some(mnemonic) => stronghold.account_import(
                created_at_timestamp,
                created_at_timestamp,
                mnemonic,
                Some("password"),
                "password",
                vec![],
            ),
            None => stronghold.account_create(Some("password".to_string()), "password"),
        });

        let id = stronghold_account.id().to_string();
        let account_id: AccountIdentifier = id.clone().into();

        let account = Account {
            id,
            alias,
            created_at,
            transactions: self.transactions,
            addresses: self.addresses,
            client_options: self.client_options,
        };
        let adapter = crate::storage::get_adapter()?;
        adapter.set(account_id, serde_json::to_string(&account)?)?;
        Ok(account)
    }
}

/// Account definition.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    id: String,
    /// The account alias.
    alias: String,
    /// Time of account creation.
    created_at: DateTime<Utc>,
    /// Transactions associated with the seed.
    /// The account can be initialised with locally stored transactions.
    #[serde(skip)]
    #[getset(set = "pub(crate)")]
    transactions: Vec<Transaction>,
    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    addresses: Vec<Address>,
    /// The client options.
    client_options: ClientOptions,
}

impl Account {
    /// Returns the most recent address of the account.
    pub fn latest_address(&self) -> &Address {
        &self.addresses.iter().max_by_key(|a| a.key_index()).unwrap()
    }
    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub fn sync(&self) -> AccountSynchronizer<'_> {
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
    /// For example, if a user with 50i total account balance has made a transaction spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub fn available_balance(&self) -> u64 {
        let total_balance = self.total_balance();
        let spent = self.transactions.iter().fold(0, |acc, tx| {
            let val = if *tx.confirmed() {
                0
            } else {
                tx.value().without_denomination()
            };
            acc + val
        });
        total_balance - (spent as u64)
    }

    /// Updates the account alias.
    pub fn set_alias(&mut self, alias: impl AsRef<str>) -> crate::Result<()> {
        self.alias = alias.as_ref().to_string();
        crate::storage::get_adapter()?
            .set(self.id.to_string().into(), serde_json::to_string(self)?)?;
        Ok(())
    }

    /// Gets a list of transactions on this account.
    /// It's fetched from the storage. To ensure the database is updated with the latest transactions,
    /// `sync` should be called first.
    ///
    /// * `count` - Number of (most recent) transactions to fetch.
    /// * `from` - Starting point of the subset to fetch.
    /// * `transaction_type` - Optional transaction type filter.
    ///
    /// # Example
    ///
    /// ```
    /// use iota_wallet::transaction::TransactionType;
    /// use iota_wallet::account_manager::AccountManager;
    /// use iota_wallet::client::ClientOptionsBuilder;
    ///
    /// // gets 10 received transactions, skipping the first 5 most recent transactions.
    /// let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
    ///  .expect("invalid node URL")
    ///  .build();
    /// let mut manager = AccountManager::new();
    /// let mut account = manager.create_account(client_options)
    ///   .initialise()
    ///   .expect("failed to add account");
    /// account.list_transactions(10, 5, Some(TransactionType::Received));
    /// ```
    pub fn list_transactions(
        &self,
        count: u64,
        from: u64,
        transaction_type: Option<TransactionType>,
    ) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|tx| {
                if let Some(tx_type) = transaction_type.clone() {
                    match tx_type {
                        TransactionType::Received => self.addresses.contains(tx.address()),
                        TransactionType::Sent => !self.addresses.contains(tx.address()),
                        TransactionType::Failed => !tx.broadcasted(),
                        TransactionType::Unconfirmed => !tx.confirmed(),
                        TransactionType::Value => tx.value().without_denomination() > 0,
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
        let address = crate::address::get_new_address(&self).await?;
        self.addresses.push(address.clone());
        crate::storage::get_adapter()?
            .set(self.id.to_string().into(), serde_json::to_string(self)?)?;
        Ok(address)
    }

    pub(crate) fn append_transactions(&mut self, transactions: Vec<Transaction>) {
        self.transactions.extend(transactions.iter().cloned());
    }

    /// Gets a transaction with the given hash associated with this account.
    pub fn get_transaction(&self, hash: &Hash) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.hash() == hash)
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
    transactions: Vec<Transaction>,
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
    use crate::transaction::{Tag, Transaction, TransactionType, Value, ValueUnit};

    use iota::crypto::ternary::{sponge::Kerl, Hash};
    use iota::signing::ternary::{
        seed::Seed,
        wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
        PrivateKey, PrivateKeyGenerator, PublicKey,
    };
    use iota::ternary::{T1B1Buf, TryteBuf};
    use iota::transaction::bundled::{Address as IotaAddress, BundledTransactionField};

    #[test]
    // asserts that the `set_alias` function updates the account alias in storage
    fn set_alias() {
        let manager = AccountManager::new();
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
            .get_account(account.id().to_string().into())
            .expect("failed to get account from storage");
        assert_eq!(
            account_in_storage.alias().to_string(),
            updated_alias.to_string()
        );
    }

    fn _generate_iota_address(seed: &str) -> IotaAddress {
        let seed = Seed::from_trits(
            TryteBuf::try_from_str(seed)
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>(),
        )
        .unwrap();

        IotaAddress::try_from_inner(
            WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(WotsSecurityLevel::Medium)
                .build()
                .unwrap()
                .generate_from_seed(&seed, 3)
                .unwrap()
                .generate_public_key()
                .unwrap()
                .as_trits()
                .to_owned(),
        )
        .unwrap()
    }

    fn _generate_account(transactions: Vec<Transaction>) -> (Account, Address, u64) {
        let manager = AccountManager::new();
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        let address = _generate_iota_address(
            "RVORZ9SIIP9RCYMREUIXXVPQIPHVCNPQ9HZWYKFWYWZRE9JQKG9REPKIASHUUECPSQO9JT9XNMVKWYGVA",
        );
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
        let account = manager
            .create_account(client_options)
            .alias("alias")
            .addresses(addresses)
            .transactions(transactions)
            .initialise()
            .expect("failed to add account");
        (account, second_address, balance)
    }

    fn _generate_transaction(
        value: i64,
        address: Address,
        confirmed: bool,
        broadcasted: bool,
    ) -> Transaction {
        Transaction {
            hash: Hash::zeros(),
            address,
            value: Value::new(value, ValueUnit::I),
            tag: Tag::default(),
            timestamp: chrono::Utc::now(),
            current_index: 0,
            last_index: 0,
            bundle_hash: Hash::zeros(),
            trunk_transaction: Hash::zeros(),
            branch_transaction: Hash::zeros(),
            nonce: String::default(),
            confirmed,
            broadcasted,
        }
    }

    #[test]
    fn latest_address() {
        let (account, latest_address, _) = _generate_account(vec![]);
        assert_eq!(account.latest_address(), &latest_address);
    }

    #[test]
    fn total_balance() {
        let (account, _, balance) = _generate_account(vec![]);
        assert_eq!(account.total_balance(), balance);
    }

    #[test]
    fn available_balance() {
        let (account, _, balance) = _generate_account(vec![]);
        assert_eq!(account.available_balance(), balance);

        let unconfirmed_transaction = _generate_transaction(
            15,
            account.addresses().first().unwrap().clone(),
            false,
            false,
        );
        let confirmed_transaction =
            _generate_transaction(10, account.addresses().first().unwrap().clone(), true, true);
        let (account, _, balance) =
            _generate_account(vec![unconfirmed_transaction.clone(), confirmed_transaction]);
        assert_eq!(
            account.available_balance(),
            balance - unconfirmed_transaction.value().without_denomination() as u64
        );
    }

    #[test]
    fn list_all_transactions() {
        let (mut account, _, _) = _generate_account(vec![]);
        let received_transaction =
            _generate_transaction(0, account.latest_address().clone(), true, true);
        let failed_transaction =
            _generate_transaction(0, account.latest_address().clone(), true, false);
        let unconfirmed_transaction =
            _generate_transaction(0, account.latest_address().clone(), false, true);
        let value_transaction =
            _generate_transaction(4, account.latest_address().clone(), true, true);
        account.append_transactions(vec![
            received_transaction,
            failed_transaction,
            unconfirmed_transaction,
            value_transaction,
        ]);

        let txs = account.list_transactions(4, 0, None);
        assert_eq!(txs.len(), 4);
    }

    #[test]
    fn list_transactions_by_type() {
        let (mut account, _, _) = _generate_account(vec![]);

        let external_address = AddressBuilder::new()
            .address(_generate_iota_address(
                "AVXX9XWUSUVKUTWXKTBG9BJVBTZSAISBILKJNVWUHOQNYDMQWXNUCLTTOZGTTLLIYDXXJJGJSEOKVOSSZ",
            ))
            .key_index(0)
            .balance(0)
            .build()
            .unwrap();

        let received_transaction =
            _generate_transaction(0, account.latest_address().clone(), true, true);
        let sent_transaction = _generate_transaction(0, external_address.clone(), true, true);
        let failed_transaction =
            _generate_transaction(0, account.latest_address().clone(), true, false);
        let unconfirmed_transaction =
            _generate_transaction(0, account.latest_address().clone(), false, true);
        let value_transaction =
            _generate_transaction(5, account.latest_address().clone(), true, true);
        account.append_transactions(vec![
            received_transaction.clone(),
            sent_transaction.clone(),
            failed_transaction.clone(),
            unconfirmed_transaction.clone(),
            value_transaction.clone(),
        ]);

        let cases = vec![
            (TransactionType::Failed, &failed_transaction),
            (TransactionType::Received, &received_transaction),
            (TransactionType::Sent, &sent_transaction),
            (TransactionType::Unconfirmed, &unconfirmed_transaction),
            (TransactionType::Value, &value_transaction),
        ];
        for (tx_type, expected) in cases {
            let failed_txs = account.list_transactions(1, 0, Some(tx_type.clone()));
            assert_eq!(
                failed_txs.len(),
                if tx_type == TransactionType::Received {
                    4
                } else {
                    1
                }
            );
            assert_eq!(failed_txs.first().unwrap(), &expected);
        }
    }

    #[test]
    fn get_transaction_by_hash() {
        let (mut account, _, _) = _generate_account(vec![]);
        let hash = Hash::from_inner_unchecked(
            TryteBuf::try_from_str(
                "IITL9EALLVZEGFIFBCCAHUOKHFBIIKQACBCEVVNZUEQLUJTOPXRICFRZKJDQGSVHARJANFDDAHMERS999",
            )
            .unwrap()
            .as_trits()
            .encode(),
        );

        let mut transaction =
            _generate_transaction(0, account.latest_address().clone(), true, true);
        transaction.set_hash(hash);
        account.append_transactions(vec![
            _generate_transaction(10, account.latest_address().clone(), true, true),
            transaction.clone(),
        ]);
        assert_eq!(account.get_transaction(&hash).unwrap(), &transaction);
    }

    // TODO this test needs some work on the client initialization
    /*#[tokio::test]
    async fn sync() {
      let (account, _, _) = _generate_account(vec![]);
      account.sync().execute().await.unwrap();
    }*/
    // TODO list_addresses, generate_addresses tests
}

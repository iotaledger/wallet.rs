use crate::account::Account;
use getset::Getters;
pub use iota::transaction::prelude::Address as IotaAddress;
use std::convert::TryInto;

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
    address: Option<IotaAddress>,
    balance: Option<u64>,
    key_index: Option<usize>,
    internal: bool,
}

impl AddressBuilder {
    /// Initialises a new instance of the address builder.
    pub fn new() -> AddressBuilder {
        Default::default()
    }

    /// Defines the address.
    pub fn address(mut self, address: IotaAddress) -> Self {
        self.address = Some(address);
        self
    }

    /// Sets the address balance.
    pub fn balance(mut self, balance: u64) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Sets the address key index.
    pub fn key_index(mut self, key_index: usize) -> Self {
        self.key_index = Some(key_index);
        self
    }

    /// Builds the address.
    pub fn build(self) -> crate::Result<Address> {
        let iota_address = self
            .address
            .ok_or_else(|| anyhow::anyhow!("the `address` field is required"))?;
        let checksum = generate_checksum(&iota_address)?;
        let address = Address {
            address: iota_address,
            balance: self
                .balance
                .ok_or_else(|| anyhow::anyhow!("the `balance` field is required"))?,
            key_index: self
                .key_index
                .ok_or_else(|| anyhow::anyhow!("the `key_index` field is required"))?,
            checksum,
            internal: self.internal,
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Clone, Eq)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    #[serde(skip, default = "IotaAddress::zeros")]
    address: IotaAddress,
    /// The address balance.
    balance: u64,
    /// The address key index.
    key_index: usize,
    /// The address checksum.
    checksum: String,
    /// Determines if an address is a public or an internal (change) address.
    internal: bool,
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.address() == other.address()
    }
}

pub(crate) fn get_iota_address(
    account: &Account,
    index: usize,
    internal: bool,
) -> crate::Result<IotaAddress> {
    crate::with_stronghold(|stronghold| {
        let address_str = stronghold.address_get(account.id(), index, internal);
        let iota_address = IotaAddress::from_ed25519_bytes(address_str.as_bytes().try_into()?);
        Ok(iota_address)
    })
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account, internal: bool) -> crate::Result<Address> {
    let key_index = account.addresses().len();
    let iota_address = get_iota_address(&account, key_index, internal)?;
    let balance = get_balance(&account, &iota_address).await?;
    let checksum = generate_checksum(&iota_address)?;
    let address = Address {
        address: iota_address,
        balance,
        key_index,
        checksum,
        internal,
    };
    Ok(address)
}

/// Batch address generation.
pub(crate) async fn get_addresses(
    account: &Account,
    count: usize,
    internal: bool,
) -> crate::Result<Vec<Address>> {
    let mut addresses = vec![];
    for i in 0..count {
        addresses.push(get_new_address(&account, internal).await?);
    }
    Ok(addresses)
}

/// Generates a checksum for the given address
// TODO: maybe this should be part of the crypto lib
pub(crate) fn generate_checksum(address: &IotaAddress) -> crate::Result<String> {
    Ok("".to_string())
}

async fn get_balance(account: &Account, address: &IotaAddress) -> crate::Result<u64> {
    let client = crate::client::get_client(account.client_options());
    let amount = client
        .get_addresses_balance(&[address.clone()])?
        .iter()
        .fold(0, |acc, output| output.amount);
    Ok(amount)
}

pub(crate) fn is_unspent(account: &Account, address: &IotaAddress) -> bool {
    account.messages().iter().any(|message| {
        message.value().without_denomination() < 0 && message.address().address() == address
    })
}

#[cfg(test)]
mod tests {
    use super::{Address, IotaAddress};
    use crate::account::Account;
    use crate::account_manager::AccountManager;
    use crate::client::ClientOptionsBuilder;
    use crate::transaction::{Tag, Transaction, Value, ValueUnit};

    use iota::crypto::ternary::Hash;
    use iota::ternary::TryteBuf;
    use iota::transaction::bundled::BundledTransactionField;

    fn _create_account() -> Account {
        let manager = AccountManager::new();

        let client_options = ClientOptionsBuilder::node("https://nodes.comnet.thetangle.org")
            .unwrap()
            .build();
        let account = manager
            .create_account(client_options)
            .alias("alias")
            .initialise()
            .unwrap();

        account
    }

    fn _create_address() -> IotaAddress {
        IotaAddress::from_inner_unchecked(
            TryteBuf::try_from_str(
                "XUERGHWTYRTFUYKFKXURKHMFEVLOIFTTCNTXOGLDPCZ9CJLKHROOPGNAQYFJEPGK9OKUQROUECBAVNXRY",
            )
            .unwrap()
            .as_trits()
            .encode(),
        )
    }

    fn _generate_transaction(value: i64, address: Address) -> Transaction {
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
            confirmed: true,
            broadcasted: true,
        }
    }

    #[tokio::test]
    async fn get_balance() {
        let account = _create_account();
        let address = _create_address();

        let response = super::get_balance(&account, &address).await;
        assert!(response.is_ok());
    }

    #[test]
    fn is_unspent_false() {
        let account = _create_account();
        let address = _create_address();

        let response = super::is_unspent(&account, &address);
        assert_eq!(response, false);
    }

    #[tokio::test]
    async fn is_unspent_true() {
        let mut account = _create_account();
        let address = super::get_new_address(&account).await.unwrap();
        let spent_tx = _generate_transaction(-50, address.clone());
        account.append_transactions(vec![spent_tx]);

        let response = super::is_unspent(&account, address.address());
        assert_eq!(response, true);
    }
}

use crate::account::Account;
use getset::Getters;
pub use iota::transaction::prelude::Address as IotaAddress;
use serde::{Deserialize, Serialize};
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
        let address = Address {
            address: iota_address,
            balance: self
                .balance
                .ok_or_else(|| anyhow::anyhow!("the `balance` field is required"))?,
            key_index: self
                .key_index
                .ok_or_else(|| anyhow::anyhow!("the `key_index` field is required"))?,
            internal: self.internal,
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    address: IotaAddress,
    /// The address balance.
    balance: u64,
    /// The address key index.
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    internal: bool,
}

pub(crate) fn get_iota_address(
    account: &Account,
    index: usize,
    internal: bool,
) -> crate::Result<IotaAddress> {
    crate::with_stronghold(|stronghold| {
        let address_str = stronghold.address_get(account.id(), index, internal)?;
        let iota_address = IotaAddress::from_ed25519_bytes(address_str.as_bytes().try_into()?);
        Ok(iota_address)
    })
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account, internal: bool) -> crate::Result<Address> {
    let key_index = account.addresses().len();
    let iota_address = get_iota_address(&account, key_index, internal)?;
    let balance = get_balance(&account, &iota_address).await?;
    let address = Address {
        address: iota_address,
        balance,
        key_index,
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

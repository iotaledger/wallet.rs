use crate::account::Account;
use getset::Getters;
use iota::crypto::ternary::sponge::{Kerl, Sponge};
use iota::ternary::{TritBuf, TryteBuf};
pub use iota::transaction::bundled::Address as IotaAddress;
use iota::transaction::bundled::BundledTransactionField;

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
    address: Option<IotaAddress>,
    balance: Option<u64>,
    key_index: Option<usize>,
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
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    address: IotaAddress,
    /// The address balance.
    balance: u64,
    /// The address key index.
    key_index: usize,
    /// The address checksum.
    checksum: TritBuf,
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.key_index() == other.key_index()
    }
}

pub(crate) fn get_new_iota_address(account: &Account) -> crate::Result<(usize, IotaAddress)> {
    let (key_index, iota_address) = crate::with_stronghold(|stronghold| {
        let (address_index, address_str) =
            stronghold.address_get(account.id().as_str(), 0, false, "password");
        let iota_address = IotaAddress::from_inner_unchecked(
            TryteBuf::try_from_str(&address_str)
                .expect("failed to get TryteBuf from address")
                .as_trits()
                .encode(),
        );
        (address_index, iota_address)
    });
    Ok((key_index, iota_address))
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account) -> crate::Result<Address> {
    let (key_index, iota_address) = get_new_iota_address(&account)?;
    let balance = get_balance(&account, &iota_address).await?;
    let checksum = generate_checksum(&iota_address)?;
    let address = Address {
        address: iota_address,
        balance,
        key_index,
        checksum,
    };
    Ok(address)
}

/// Batch address generation.
pub(crate) async fn get_addresses(account: &Account, count: usize) -> crate::Result<Vec<Address>> {
    let mut addresses = vec![];
    for i in 0..count {
        let (index, address) = crate::with_stronghold(|stronghold| {
            let (address_index, address_str) =
                stronghold.address_get(account.id().as_str(), 0, false, "password");
            let iota_address = IotaAddress::from_inner_unchecked(
                TryteBuf::try_from_str(&address_str)
                    .expect("failed to get TryteBuf from address")
                    .as_trits()
                    .encode(),
            );
            (address_index, iota_address)
        });
        let balance = get_balance(&account, &address).await?;
        let checksum = generate_checksum(&address)?;
        addresses.push(Address {
            address,
            balance,
            key_index: index,
            checksum,
        })
    }
    Ok(addresses)
}

/// Generates a checksum for the given address
// TODO: maybe this should be part of the crypto lib
pub(crate) fn generate_checksum(address: &IotaAddress) -> crate::Result<TritBuf> {
    let mut kerl = Kerl::new();
    let mut hash = kerl
        .digest(address.to_inner())
        .map_err(|e| anyhow::anyhow!("Erro hashing the address"))?;
    let mut trits = vec![];

    for _ in 1..10 {
        if let Some(trit) = hash.pop() {
            trits.push(trit);
        } else {
            return Err(anyhow::anyhow!("Hash error"));
        }
    }

    Ok(TritBuf::from_trits(&trits[..]))
}

async fn get_balance(account: &Account, address: &IotaAddress) -> crate::Result<u64> {
    let client = crate::client::get_client(account.client_options());
    client
        .get_balances()
        .addresses(&[address.clone()])
        .send()
        .await?
        .balances
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("Balances response empty"))
}

pub(crate) fn is_unspent(account: &Account, address: &IotaAddress) -> bool {
    account
        .transactions()
        .iter()
        .any(|tx| tx.value().without_denomination() < 0 && tx.address().address() == address)
}

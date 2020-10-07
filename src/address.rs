use crate::account::Account;
use bech32::FromBase32;
use getset::Getters;
pub use iota::transaction::prelude::{
    Address as IotaAddress, Ed25519Address, SignatureLockedSingleOutput, UTXOInput,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

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
#[derive(Debug, Getters, Clone, Eq, Serialize, Deserialize)]
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

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.to_bech32().cmp(&other.address.to_bech32())
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.to_bech32().hash(state);
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address.to_bech32() == other.address.to_bech32()
    }
}

pub(crate) fn get_iota_address(
    account_id: &[u8; 32],
    account_index: usize,
    address_index: usize,
    internal: bool,
) -> crate::Result<IotaAddress> {
    crate::with_stronghold(|stronghold| {
        let address_str =
            stronghold.address_get(account_id, Some(account_index), address_index, internal)?;
        let address_ed25519 = Vec::from_base32(&bech32::decode(&address_str)?.1)?;
        let iota_address =
            IotaAddress::Ed25519(Ed25519Address::new(address_ed25519[1..].try_into()?));
        Ok(iota_address)
    })
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account, internal: bool) -> crate::Result<Address> {
    let key_index = account.addresses().len();
    let iota_address = get_iota_address(account.id(), account.index()?, key_index, internal)?;
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

#[cfg(test)]
mod tests {
    use super::{Address, IotaAddress};
    use crate::account::Account;
    use crate::account_manager::AccountManager;
    use crate::client::ClientOptionsBuilder;
    use crate::message::Message;

    use chrono::Utc;
    use iota::transaction::prelude::{
        Ed25519Address, MessageId, Payload, Seed, SignatureLockedSingleOutput, TransactionBuilder,
        UTXOInput,
    };
    use slip10::path::BIP32Path;
    use std::convert::TryInto;
    use std::num::NonZeroU64;

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
        IotaAddress::Ed25519(Ed25519Address::new([0; 32]))
    }

    fn _generate_message(value: i64, address: Address) -> Message {
        Message {
            version: 1,
            trunk: MessageId::new([0; 32]),
            branch: MessageId::new([0; 32]),
            payload_length: 0,
            payload: Payload::Transaction(Box::new(
                TransactionBuilder::new(&Seed::from_ed25519_bytes("".as_bytes()).unwrap())
                    .set_outputs(vec![SignatureLockedSingleOutput::new(
                        address.address().clone(),
                        NonZeroU64::new(value.try_into().unwrap()).unwrap(),
                    )
                    .into()])
                    .set_inputs(vec![(
                        UTXOInput::new(MessageId::new([0; 32]), 0).unwrap().into(),
                        BIP32Path::from_str("").unwrap(),
                    )])
                    .build()
                    .unwrap(),
            )),
            timestamp: Utc::now(),
            nonce: 0,
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
        let address = super::get_new_address(&account, false).await.unwrap();
        let spent_tx = _generate_message(-50, address.clone());
        account.append_messages(vec![spent_tx]);

        let response = super::is_unspent(&account, address.address());
        assert_eq!(response, true);
    }
}

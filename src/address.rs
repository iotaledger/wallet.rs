// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, message::MessageType, signing::GenerateAddressMetadata};
use getset::{Getters, Setters};
use iota::{
    message::prelude::{MessageId, TransactionId},
    OutputMetadata,
};
pub use iota::{Address as IotaAddress, Ed25519Address, Input, Payload, UTXOInput};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    hash::{Hash, Hasher},
    str::FromStr,
};

/// An Address output.
#[derive(Debug, Getters, Setters, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AddressOutput {
    /// Transaction ID of the output
    transaction_id: TransactionId,
    /// Message ID of the output
    message_id: MessageId,
    /// Output index.
    index: u16,
    /// Output amount.
    amount: u64,
    /// Spend status of the output,
    is_spent: bool,
}

impl AddressOutput {
    /// Checks if the output is referenced on a pending message or a confirmed message
    pub(crate) fn is_used(&self, account: &Account) -> bool {
        let output_id = UTXOInput::new(self.transaction_id, self.index).unwrap();
        account.list_messages(0, 0, None).iter().any(|m| {
            // message is pending or confirmed
            if m.confirmed().unwrap_or(true) {
                match m.payload() {
                    Payload::Transaction(tx) => tx.essence().inputs().iter().any(|input| {
                        if let Input::UTXO(x) = input {
                            x == &output_id
                        } else {
                            false
                        }
                    }),
                    _ => false,
                }
            } else {
                false
            }
        })
    }
}

impl TryFrom<OutputMetadata> for AddressOutput {
    type Error = crate::Error;

    fn try_from(output: OutputMetadata) -> crate::Result<Self> {
        let output = Self {
            transaction_id: TransactionId::new(
                output.transaction_id[..]
                    .try_into()
                    .map_err(|_| crate::Error::InvalidTransactionId)?,
            ),
            message_id: MessageId::new(
                output.message_id[..]
                    .try_into()
                    .map_err(|_| crate::Error::InvalidMessageId)?,
            ),
            index: output.output_index,
            amount: output.amount,
            is_spent: output.is_spent,
        };
        Ok(output)
    }
}

/// The address builder.
#[derive(Default)]
pub(crate) struct AddressBuilder {
    address: Option<AddressWrapper>,
    balance: Option<u64>,
    key_index: Option<usize>,
    internal: bool,
    outputs: Option<Vec<AddressOutput>>,
}

impl AddressBuilder {
    /// Initialises a new instance of the address builder.
    pub fn new() -> AddressBuilder {
        Default::default()
    }

    /// Defines the address.
    pub fn address(mut self, address: AddressWrapper) -> Self {
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

    /// Sets the address outputs.
    pub fn outputs(mut self, outputs: Vec<AddressOutput>) -> Self {
        self.outputs = Some(outputs);
        self
    }

    /// Sets the `internal` flag.
    pub fn internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }

    /// Builds the address.
    pub fn build(self) -> crate::Result<Address> {
        let iota_address = self.address.ok_or(crate::Error::AddressBuildRequiredField(
            crate::error::AddressBuildRequiredField::Address,
        ))?;
        let address = Address {
            address: iota_address,
            balance: self.balance.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::Balance,
            ))?,
            key_index: self.key_index.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::KeyIndex,
            ))?,
            internal: self.internal,
            outputs: self.outputs.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::Outputs,
            ))?,
        };
        Ok(address)
    }
}

/// An address and its network type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressWrapper {
    inner: IotaAddress,
    hrp: String,
}

impl AsRef<IotaAddress> for AddressWrapper {
    fn as_ref(&self) -> &IotaAddress {
        &self.inner
    }
}

impl AddressWrapper {
    pub(crate) fn new(address: IotaAddress, hrp: String) -> Self {
        Self { inner: address, hrp }
    }

    /// Encodes the address as bech32.
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32(&self.hrp)
    }
}

/// An address.
#[derive(Debug, Getters, Setters, Clone, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The address balance.
    #[getset(set = "pub")]
    balance: u64,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    internal: bool,
    /// The address outputs.
    #[getset(set = "pub(crate)")]
    pub(crate) outputs: Vec<AddressOutput>,
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

impl Address {
    pub(crate) fn handle_new_output(&mut self, output: AddressOutput) {
        if !self.outputs.iter().any(|o| o == &output) {
            let spent_existing_output = self.outputs.iter().position(|o| {
                o.message_id == output.message_id
                    && o.transaction_id == output.transaction_id
                    && o.index == output.index
                    && o.amount == output.amount
                    && (!o.is_spent && output.is_spent)
            });
            if let Some(spent_output) = spent_existing_output {
                log::debug!("[ADDRESS] got spent of {:?}", spent_output);
                self.balance -= output.amount;
                self.outputs.remove(spent_output);
            } else {
                log::debug!("[ADDRESS] got new output {:?}", output);
                self.balance += output.amount;
                self.outputs.push(output);
            }
        }
    }

    /// Gets the list of outputs that aren't spent or pending.
    pub fn available_outputs(&self, account: &Account) -> Vec<&AddressOutput> {
        self.outputs.iter().filter(|o| !o.is_used(account)).collect()
    }

    pub(crate) fn available_balance(&self, account: &Account) -> u64 {
        self.available_outputs(account)
            .iter()
            .fold(0, |acc, o| acc + *o.amount())
    }

    pub(crate) fn set_bech32_hrp(&mut self, hrp: String) {
        self.address.hrp = hrp;
    }
}

/// Parses a bech32 address string.
pub fn parse<A: AsRef<str>>(address: A) -> crate::Result<AddressWrapper> {
    let address = address.as_ref();
    let mut tokens = address.split('1');
    let hrp = tokens.next().unwrap();
    let address = iota::Address::try_from_bech32(address).or_else(|_| {
        if let Ok(ed25519_address) = Ed25519Address::from_str(address) {
            Ok(IotaAddress::Ed25519(ed25519_address))
        } else {
            Err(crate::Error::InvalidAddress)
        }
    });
    Ok(AddressWrapper::new(address?, hrp.to_string()))
}

pub(crate) async fn get_iota_address(
    account: &Account,
    address_index: usize,
    internal: bool,
    bech32_hrp: String,
    metadata: GenerateAddressMetadata,
) -> crate::Result<AddressWrapper> {
    let signer = crate::signing::get_signer(account.signer_type()).await;
    let mut signer = signer.lock().await;
    let address = signer
        .generate_address(&account, address_index, internal, metadata)
        .await?;
    Ok(AddressWrapper::new(address, bech32_hrp))
}

/// Gets an unused public address for the given account.
pub(crate) async fn get_new_address(account: &Account, metadata: GenerateAddressMetadata) -> crate::Result<Address> {
    let key_index = account.addresses().iter().filter(|a| !a.internal()).count();
    let bech32_hrp = match account.addresses().first() {
        Some(address) => address.address().hrp.to_string(),
        None => {
            crate::client::get_client(account.client_options())
                .read()
                .await
                .get_network_info()
                .bech32_hrp
        }
    };
    let iota_address = get_iota_address(&account, key_index, false, bech32_hrp, metadata).await?;
    let address = Address {
        address: iota_address,
        balance: 0,
        key_index,
        internal: false,
        outputs: vec![],
    };
    Ok(address)
}

/// Gets an unused change address for the given account and address.
pub(crate) async fn get_new_change_address(
    account: &Account,
    address: &Address,
    metadata: GenerateAddressMetadata,
) -> crate::Result<Address> {
    let key_index = *address.key_index();
    let iota_address = get_iota_address(&account, key_index, true, address.address().hrp.to_string(), metadata).await?;
    let address = Address {
        address: iota_address,
        balance: 0,
        key_index,
        internal: true,
        outputs: vec![],
    };
    Ok(address)
}

pub(crate) fn is_unspent(account: &Account, address: &AddressWrapper) -> bool {
    !account
        .list_messages(0, 0, Some(MessageType::Sent))
        .iter()
        .any(|message| message.addresses().contains(&address.as_ref()))
}

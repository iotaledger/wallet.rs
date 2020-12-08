// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, message::MessageType};
use bech32::FromBase32;
use getset::{Getters, Setters};
pub use iota::message::prelude::{Address as IotaAddress, Ed25519Address};
use iota::{
    message::prelude::{MessageId, TransactionId},
    OutputMetadata,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    hash::{Hash, Hasher},
};

/// An Address output.
#[derive(Debug, Getters, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Spend status of the output.
    is_spent: bool,
}

impl TryFrom<OutputMetadata> for AddressOutput {
    type Error = crate::WalletError;

    fn try_from(output: OutputMetadata) -> crate::Result<Self> {
        let output = Self {
            transaction_id: TransactionId::new(
                output.transaction_id[..]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid transaction id length"))?,
            ),
            message_id: MessageId::new(
                output.message_id[..]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid message id length"))?,
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
pub struct AddressBuilder {
    address: Option<IotaAddress>,
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
            outputs: self
                .outputs
                .ok_or_else(|| anyhow::anyhow!("the `outputs` field is required"))?,
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Setters, Clone, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: IotaAddress,
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
    outputs: Vec<AddressOutput>,
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
                self.balance -= output.amount;
                self.outputs.remove(spent_output);
            } else {
                self.balance += output.amount;
                self.outputs.push(output);
            }
        }
    }
}

/// Parses a bech32 address string.
pub fn parse(address: String) -> crate::Result<IotaAddress> {
    let address_ed25519 = Vec::from_base32(&bech32::decode(&address)?.1)?;
    let iota_address = IotaAddress::Ed25519(Ed25519Address::new(
        address_ed25519[1..]
            .try_into()
            .map_err(|_| crate::WalletError::InvalidAddressLength)?,
    ));
    Ok(iota_address)
}

pub(crate) fn get_iota_address(account: &Account, address_index: usize, internal: bool) -> crate::Result<IotaAddress> {
    crate::signing::with_signer(account.signer_type(), |signer| {
        signer.generate_address(&account, address_index, internal)
    })
}

/// Gets an unused public address for the given account.
pub(crate) fn get_new_address(account: &Account) -> crate::Result<Address> {
    let key_index = account.addresses().iter().filter(|a| !a.internal()).count();
    let iota_address = get_iota_address(&account, key_index, false)?;
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
pub(crate) fn get_new_change_address(account: &Account, address: &Address) -> crate::Result<Address> {
    let key_index = *address.key_index();
    let iota_address = get_iota_address(&account, key_index, true)?;
    let address = Address {
        address: iota_address,
        balance: 0,
        key_index,
        internal: true,
        outputs: vec![],
    };
    Ok(address)
}

/// Batch address generation.
pub(crate) fn get_addresses(account: &Account, count: usize) -> crate::Result<Vec<Address>> {
    let mut addresses = vec![];
    for i in 0..count {
        addresses.push(get_new_address(&account)?);
    }
    Ok(addresses)
}

pub(crate) fn is_unspent(account: &Account, address: &IotaAddress) -> bool {
    !account
        .list_messages(0, 0, Some(MessageType::Sent))
        .iter()
        .any(|message| message.addresses().contains(&address))
}

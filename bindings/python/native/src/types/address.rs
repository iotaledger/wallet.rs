// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_wallet::address::{
    Address as RustWalletAddress, AddressOutput as RustAddressOutput, AddressWrapper as RustAddressWrapper,
};
use std::{
    collections::HashMap,
    convert::{From, Into},
};

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Address {
    /// The address.
    address: AddressWrapper,
    /// The address balance.
    balance: u64,
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    internal: bool,
    /// The address outputs.
    outputs: HashMap<String, AddressOutput>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct AddressWrapper {
    inner: String,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct AddressOutput {
    /// Transaction ID of the output
    transaction_id: String,
    /// Message ID of the output
    message_id: String,
    /// Output index.
    index: u16,
    /// Output amount.
    amount: u64,
    /// Spend status of the output,
    is_spent: bool,
    /// Associated address.
    address: AddressWrapper,
}

impl From<&RustAddressOutput> for AddressOutput {
    fn from(output: &RustAddressOutput) -> Self {
        Self {
            transaction_id: output.transaction_id().to_string(),
            message_id: output.message_id().to_string(),
            index: *output.index(),
            amount: *output.amount(),
            is_spent: *output.is_spent(),
            address: output.address().into(),
        }
    }
}

impl From<&RustAddressWrapper> for AddressWrapper {
    fn from(wrapper: &RustAddressWrapper) -> Self {
        Self {
            inner: wrapper.to_bech32(),
        }
    }
}

impl From<RustAddressWrapper> for AddressWrapper {
    fn from(wrapper: RustAddressWrapper) -> Self {
        Self {
            inner: wrapper.to_bech32(),
        }
    }
}

impl From<RustWalletAddress> for Address {
    fn from(wallet_address: RustWalletAddress) -> Self {
        Self {
            address: wallet_address.address().into(),
            balance: wallet_address.balance(),
            key_index: *wallet_address.key_index(),
            internal: *wallet_address.internal(),
            outputs: wallet_address
                .outputs()
                .iter()
                .map(|(id, output)| (id.to_string(), output.into()))
                .collect(),
        }
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;

use getset::{Getters, Setters};
use iota_client::block::{address::Address, output::OutputId};
use serde::{Deserialize, Serialize};

/// An account address.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[getset(get = "pub")]
pub struct AccountAddress {
    /// The address.
    #[serde(with = "crate::account::types::address_serde")]
    pub(crate) address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    pub(crate) key_index: u32,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    pub(crate) internal: bool,
    // do we want this field? Could be useful if we don't store spent output ids and because of that wouldn't know if
    // an address was used or not just by looking at it
    pub(crate) used: bool,
}

/// An account address with unspent output_ids for unspent outputs.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[getset(get = "pub")]
pub struct AddressWithUnspentOutputs {
    /// The address.
    #[serde(with = "crate::account::types::address_serde")]
    pub(crate) address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    pub(crate) key_index: u32,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    pub(crate) internal: bool,
    /// Output ids
    #[serde(rename = "outputIds")]
    pub(crate) output_ids: Vec<OutputId>,
}

/// An address and its network type.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AddressWrapper {
    pub(crate) inner: Address,
    #[serde(rename = "bech32Hrp")]
    pub(crate) bech32_hrp: String,
}

impl AsRef<Address> for AddressWrapper {
    fn as_ref(&self) -> &Address {
        &self.inner
    }
}

impl AddressWrapper {
    /// Create a new address wrapper.
    pub fn new(address: Address, bech32_hrp: String) -> Self {
        Self {
            inner: address,
            bech32_hrp,
        }
    }

    /// Encodes the address as bech32.
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32(&self.bech32_hrp)
    }

    /// Get the bech32 human readable part
    pub fn bech32_hrp(&self) -> &str {
        &self.bech32_hrp
    }
}

/// Parses a bech32 address string.
pub fn parse_bech32_address<A: AsRef<str>>(address: A) -> crate::Result<AddressWrapper> {
    let (bech32_hrp, address) = Address::try_from_bech32(address)?;
    Ok(AddressWrapper::new(address, bech32_hrp))
}

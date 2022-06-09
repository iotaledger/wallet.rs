// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;

use getset::{Getters, Setters};
use iota_client::bee_block::output::OutputId;
use serde::{Deserialize, Serialize};

/// An account address.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[getset(get = "pub")]
pub struct AccountAddress {
    /// The address.
    pub(crate) address: String,
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
    pub(crate) address: String,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    pub(crate) key_index: u32,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    pub(crate) internal: bool,
    /// Amount
    pub(crate) amount: u64,
    /// Output ids
    #[serde(rename = "outputIds")]
    pub(crate) output_ids: Vec<OutputId>,
}

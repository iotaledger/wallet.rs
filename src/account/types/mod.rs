// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Address types used in the account
pub(crate) mod address;
/// Custom de/serialization for [`address::AddressWrapper`]
pub(crate) mod address_serde;
use crate::account::constants::ACCOUNT_ID_PREFIX;

use iota_client::bee_message::{
    address::Address, output::OutputId, payload::transaction::TransactionPayload, MessageId,
};
use serde::{Deserialize, Deserializer, Serialize};

use std::str::FromStr;

/// The balance of an account, returned from [`crate::account::handle::AccountHandle::sync()`] and
/// [`crate::account::handle::AccountHandle::balance()`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    pub(crate) total: u64,
    pub(crate) available: u64,
}

/// An output with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputData {
    /// The output id
    #[serde(rename = "outputId")]
    pub output_id: OutputId,
    /// Message ID
    #[serde(rename = "messageId")]
    pub message_id: MessageId,
    pub amount: u64,
    /// If an output is spent
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    /// Associated address.
    pub address: Address,
    pub kind: OutputKind,
    /// Network ID
    #[serde(rename = "networkId")]
    pub network_id: u64,
    // get it from the milestone that confirmed it (get metadata of the oupt and calculate the time backwards if the
    // milestone was pruned)
    pub timestamp: u128,
    pub remainder: bool,
}

/// A transaction with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub payload: TransactionPayload,
    pub message_id: Option<MessageId>,
    pub inclusion_state: InclusionState,
    pub timestamp: u128,
    // network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: u64,
    // set if the transaction was created by the wallet or if it was sent by someone else and is incoming
    pub incoming: bool,
    // do we want this field? could be used for internal transfers later, but not really necessary
    pub internal: bool,
}

/// Possible InclusionStates for transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InclusionState {
    Pending,
    Confirmed,
    Conflicting,
}

/// The output kind enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputKind {
    /// SignatureLockedSingle output.
    SignatureLockedSingle,
    /// Dust allowance output.
    SignatureLockedDustAllowance,
    /// Treasury output.
    Treasury,
}

impl FromStr for OutputKind {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "SignatureLockedSingle" => Self::SignatureLockedSingle,
            "SignatureLockedDustAllowance" => Self::SignatureLockedDustAllowance,
            "Treasury" => Self::Treasury,
            _ => return Err(crate::Error::InvalidOutputKind(s.to_string())),
        };
        Ok(kind)
    }
}

/// The account identifier.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AccountIdentifier {
    // SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in
    // Stronghold. The id should be provided by Stronghold. can we do the hashing only during interaction with
    // Stronghold? Then we could use the first address instead which could be useful
    Id(String),
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(usize),
}

impl<'de> Deserialize<'de> for AccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(AccountIdentifier::from(s))
    }
}

// When the identifier is a string id.
impl From<&str> for AccountIdentifier {
    fn from(value: &str) -> Self {
        if value.starts_with(ACCOUNT_ID_PREFIX) {
            Self::Id(value.to_string())
        } else {
            Self::Alias(value.to_string())
        }
    }
}

impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for AccountIdentifier {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

// When the identifier is an index.
impl From<usize> for AccountIdentifier {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

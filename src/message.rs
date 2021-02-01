// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::{Address, AddressWrapper, IotaAddress};
use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
pub use iota::{common::packable::Packable, IndexationPayload, Message as IotaMessage, MessageId, Output, Payload};
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    num::NonZeroU64,
    unimplemented,
};

/// The strategy to use for the remainder value management when sending funds.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum RemainderValueStrategy {
    /// Keep the remainder value on the source address.
    ReuseAddress,
    /// Move the remainder value to a change address.
    ChangeAddress,
    /// Move the remainder value to an address that must belong to the source account.
    #[serde(with = "crate::serde::iota_address_serde")]
    AccountAddress(AddressWrapper),
}

impl Default for RemainderValueStrategy {
    fn default() -> Self {
        Self::ChangeAddress
    }
}

/// A transfer to make a transaction.
#[derive(Debug, Clone)]
pub struct TransferBuilder {
    /// The transfer value.
    amount: NonZeroU64,
    /// The transfer address.
    address: AddressWrapper,
    /// (Optional) message indexation.
    indexation: Option<IndexationPayload>,
    /// The strategy to use for the remainder value.
    remainder_value_strategy: RemainderValueStrategy,
}

impl<'de> Deserialize<'de> for TransferBuilder {
    fn deserialize<D>(deserializer: D) -> Result<TransferBuilder, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The message's index builder.
        #[derive(Debug, Clone, Deserialize)]
        struct IndexationPayloadBuilder {
            index: String,
            data: Option<Vec<u8>>,
        }

        impl IndexationPayloadBuilder {
            /// Builds the indexation.
            pub fn finish(self) -> crate::Result<IndexationPayload> {
                let indexation = IndexationPayload::new(self.index, &self.data.unwrap_or_default())?;
                Ok(indexation)
            }
        }

        #[derive(Debug, Clone, Deserialize)]
        pub struct TransferBuilderWrapper {
            /// The transfer value.
            amount: NonZeroU64,
            /// The transfer address.
            #[serde(with = "crate::serde::iota_address_serde")]
            address: AddressWrapper,
            /// (Optional) message indexation.
            indexation: Option<IndexationPayloadBuilder>,
            /// The strategy to use for the remainder value.
            remainder_value_strategy: RemainderValueStrategy,
        }

        TransferBuilderWrapper::deserialize(deserializer).and_then(|builder| {
            Ok(TransferBuilder {
                amount: builder.amount,
                address: builder.address,
                indexation: match builder.indexation {
                    Some(i) => Some(i.finish().map_err(serde::de::Error::custom)?),
                    None => None,
                },
                remainder_value_strategy: builder.remainder_value_strategy,
            })
        })
    }
}

impl TransferBuilder {
    /// Initialises a new transfer to the given address.
    pub fn new(address: AddressWrapper, amount: NonZeroU64) -> Self {
        Self {
            address,
            amount,
            indexation: None,
            remainder_value_strategy: RemainderValueStrategy::ChangeAddress,
        }
    }

    /// Sets the remainder value strategy for the transfer.
    pub fn with_remainder_value_strategy(mut self, strategy: RemainderValueStrategy) -> Self {
        self.remainder_value_strategy = strategy;
        self
    }

    /// (Optional) message indexation.
    pub fn with_indexation(mut self, indexation: IndexationPayload) -> Self {
        self.indexation = Some(indexation);
        self
    }

    /// Builds the transfer.
    pub fn finish(self) -> Transfer {
        Transfer {
            address: self.address,
            amount: self.amount,
            indexation: self.indexation,
            remainder_value_strategy: self.remainder_value_strategy,
        }
    }
}

/// A transfer to make a transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct Transfer {
    /// The transfer value.
    pub(crate) amount: NonZeroU64,
    /// The transfer address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub(crate) address: AddressWrapper,
    /// (Optional) message indexation.
    pub(crate) indexation: Option<IndexationPayload>,
    /// The strategy to use for the remainder value.
    pub(crate) remainder_value_strategy: RemainderValueStrategy,
}

impl Transfer {
    /// Initialises the transfer builder.
    pub fn builder(address: AddressWrapper, amount: NonZeroU64) -> TransferBuilder {
        TransferBuilder::new(address, amount)
    }
}

/// Possible Value units.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ValueUnit {
    /// i
    I,
    /// Ki
    Ki,
    /// Mi
    Mi,
    /// Gi
    Gi,
    /// Ti
    Ti,
    /// Pi
    Pi,
}

impl fmt::Display for ValueUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ValueUnit::I => write!(f, "i"),
            ValueUnit::Ki => write!(f, "Ki"),
            ValueUnit::Mi => write!(f, "Mi"),
            ValueUnit::Gi => write!(f, "Gi"),
            ValueUnit::Ti => write!(f, "Ti"),
            ValueUnit::Pi => write!(f, "Pi"),
        }
    }
}

/// The transaction Value struct.
#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Value {
    /// The value.
    value: u64,
    /// The value's unit.
    unit: ValueUnit,
}

impl Value {
    /// Ititialises a new Value.
    pub fn new(value: u64, unit: ValueUnit) -> Self {
        Self { value, unit }
    }

    /// Formats the value with its unit.
    pub fn with_denomination(&self) -> String {
        format!("{} {}", self.value, self.unit)
    }

    /// The transaction value without its unit.
    pub fn without_denomination(&self) -> u64 {
        let multiplier = match self.unit {
            ValueUnit::I => 1,
            ValueUnit::Ki => 1000,
            ValueUnit::Mi => 1000000,
            ValueUnit::Gi => 1000000000,
            ValueUnit::Ti => 1000000000000,
            ValueUnit::Pi => 1000000000000000,
        };
        self.value * multiplier
    }
}

/// A message definition.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize, Eq)]
#[getset(get = "pub", set = "pub(crate)")]
// Need to use pub to initilize the Message structure
pub struct Message {
    /// The message identifier.
    pub id: MessageId,
    /// The message version.
    pub version: u64,
    /// Message id of the first message this message refers to.
    pub parent1: MessageId,
    /// Message id of the second message this message refers to.
    pub parent2: MessageId,
    /// Length of the payload.
    #[serde(rename = "payloadLength")]
    pub payload_length: usize,
    /// Message payload.
    pub payload: Payload,
    /// The transaction timestamp.
    pub timestamp: DateTime<Utc>,
    /// Transaction nonce.
    pub nonce: u64,
    /// Whether the transaction is confirmed or not.
    #[getset(set = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed: Option<bool>,
    /// Whether the transaction is broadcasted or not.
    #[getset(set = "pub")]
    pub broadcasted: bool,
    /// Whether the message represents an incoming transaction or not.
    pub incoming: bool,
    /// The message's value.
    pub value: u64,
    /// The message's remainder value sum.
    pub remainder_value: u64,
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.as_ref().cmp(&other.id.as_ref())
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Message {
    pub(crate) fn from_iota_message(
        id: MessageId,
        account_addresses: &[Address],
        message: &IotaMessage,
        confirmed: Option<bool>,
    ) -> crate::Result<Self> {
        let mut packed_payload = Vec::new();
        let _ = message.payload().pack(&mut packed_payload);

        let total_value = match message.payload().as_ref() {
            Some(Payload::Transaction(tx)) => tx.essence().outputs().iter().fold(0, |acc, output| {
                acc + match output {
                    Output::SignatureLockedDustAllowance(o) => o.amount(),
                    Output::SignatureLockedSingle(o) => o.amount(),
                    _ => 0,
                }
            }),
            _ => 0,
        };
        let value = Self::compute_value(&message, &id, &account_addresses).without_denomination();

        let message = Self {
            id,
            version: 1,
            parent1: *message.parent1(),
            parent2: *message.parent2(),
            payload_length: packed_payload.len(),
            payload: message.payload().as_ref().unwrap().clone(),
            timestamp: Utc::now(),
            nonce: message.nonce(),
            confirmed,
            broadcasted: true,
            incoming: account_addresses
                .iter()
                .any(|address| address.outputs().iter().any(|o| o.message_id() == &id)),
            value,
            remainder_value: total_value - value,
        };

        Ok(message)
    }

    /// The message's addresses.
    pub fn addresses(&self) -> Vec<&IotaAddress> {
        match &self.payload {
            Payload::Transaction(tx) => tx
                .essence()
                .outputs()
                .iter()
                .map(|output| match output {
                    Output::SignatureLockedDustAllowance(o) => o.address(),
                    Output::SignatureLockedSingle(o) => o.address(),
                    _ => unimplemented!(),
                })
                .collect(),
            _ => vec![],
        }
    }

    /// Gets the absolute value of the transaction.
    pub fn compute_value(iota_message: &IotaMessage, id: &MessageId, account_addresses: &[Address]) -> Value {
        let amount = match iota_message.payload().as_ref().unwrap() {
            Payload::Transaction(tx) => {
                let sent = !account_addresses
                    .iter()
                    .any(|address| address.outputs().iter().any(|o| o.message_id() == id));
                tx.essence().outputs().iter().fold(0, |acc, output| {
                    let (address, amount) = match output {
                        Output::SignatureLockedDustAllowance(o) => (o.address(), o.amount()),
                        Output::SignatureLockedSingle(o) => (o.address(), o.amount()),
                        _ => unimplemented!(),
                    };
                    let address_belongs_to_account = account_addresses.iter().any(|a| a.address().as_ref() == address);
                    if sent {
                        if address_belongs_to_account {
                            acc
                        } else {
                            acc + amount
                        }
                    } else if address_belongs_to_account {
                        acc + amount
                    } else {
                        acc
                    }
                })
            }
            _ => 0,
        };
        Value::new(amount, ValueUnit::I)
    }
}

/// Message type.
#[derive(Debug, Clone, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    /// Message received.
    Received = 1,
    /// Message sent.
    Sent = 2,
    /// Message not broadcasted.
    Failed = 3,
    /// Message not confirmed.
    Unconfirmed = 4,
    /// A value message.
    Value = 5,
}

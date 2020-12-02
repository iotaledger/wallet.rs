// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::address::{Address, IotaAddress};
use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
pub use iota::message::prelude::{Message as IotaMessage, MessageId, Output, Payload};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A transaction tag.
#[derive(Debug, Clone)]
pub struct Tag {
    tag: [u8; 16],
}

impl Default for Tag {
    /// Initialises an empty tag.
    fn default() -> Self {
        Self { tag: [0; 16] }
    }
}

impl Tag {
    /// Initialises a new tag.
    pub fn new(tag: [u8; 16]) -> Self {
        Self { tag }
    }

    /// Returns the tag formatted as ASCII.
    pub fn as_ascii(&self) -> String {
        String::from_utf8_lossy(&self.tag).to_string()
    }

    /// Returns the tag bytes.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.tag
    }
}

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
    AccountAddress(IotaAddress),
}

/// A transfer to make a transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct Transfer {
    /// The transfer value.
    pub(crate) amount: u64,
    /// The transfer address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub(crate) address: IotaAddress,
    /// (Optional) transfer data.
    pub(crate) data: Option<String>,
    /// The strategy to use for the remainder value.
    pub(crate) remainder_value_strategy: RemainderValueStrategy,
}

impl Transfer {
    /// Initialises a new transfer to the given address.
    pub fn new(address: IotaAddress, amount: u64) -> Self {
        Self {
            address,
            amount,
            data: None,
            remainder_value_strategy: RemainderValueStrategy::ChangeAddress,
        }
    }

    /// Sets the remainder value strategy for the transfer.
    pub fn remainder_value_strategy(mut self, strategy: RemainderValueStrategy) -> Self {
        self.remainder_value_strategy = strategy;
        self
    }

    /// (Optional) transfer data.
    pub fn data(mut self, data: String) -> Self {
        self.data = Some(data);
        self
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
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub(crate)")]
pub struct Message {
    /// The message identifier.
    #[serde(with = "crate::serde::message_id_serde")]
    pub(crate) id: MessageId,
    /// The message version.
    pub(crate) version: u64,
    /// Message id of the first message this message refers to.
    #[serde(with = "crate::serde::message_id_serde")]
    pub(crate) trunk: MessageId,
    /// Message id of the second message this message refers to.
    #[serde(with = "crate::serde::message_id_serde")]
    pub(crate) branch: MessageId,
    /// Length of the payload.
    #[serde(rename = "payloadLength")]
    pub(crate) payload_length: u64,
    /// Transaction amount.
    pub(crate) payload: Payload,
    /// The transaction timestamp.
    pub(crate) timestamp: DateTime<Utc>,
    /// Transaction nonce.
    pub(crate) nonce: u64,
    /// Whether the transaction is confirmed or not.
    #[getset(set = "pub")]
    pub(crate) confirmed: bool,
    /// Whether the transaction is broadcasted or not.
    #[getset(set = "pub")]
    pub(crate) broadcasted: bool,
    /// Whether the message represents an incoming transaction or not.
    pub(crate) incoming: bool,
    /// The message's value.
    pub(crate) value: u64,
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

// TODO
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.nonce == other.nonce
    }
}
impl Eq for Message {}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nonce.cmp(&other.nonce)
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
    ) -> crate::Result<Self> {
        let message = Self {
            id,
            version: 1,
            trunk: *message.parent1(),
            branch: *message.parent2(),
            payload_length: 5, // TODO
            payload: message.payload().as_ref().unwrap().clone(),
            timestamp: Utc::now(),
            // TODO timestamp: DateTime::<Utc>::from_utc(
            //    NaiveDateTime::from_timestamp(*message.attachment_ts().to_inner() as i64, 0),
            //    Utc,
            // ),
            nonce: message.nonce(),
            confirmed: false,
            broadcasted: true,
            incoming: account_addresses
                .iter()
                .any(|address| address.outputs().iter().any(|o| o.message_id() == &id)),
            value: Self::compute_value(&message, &id, &account_addresses).without_denomination(),
        };

        Ok(message)
    }

    /// Check if attachment timestamp on transaction is above max depth (~11 minutes)
    pub(crate) fn is_above_max_depth(&self) -> bool {
        let current_timestamp = Utc::now().timestamp();
        let attachment_timestamp = self.timestamp.timestamp();
        attachment_timestamp < current_timestamp
            && current_timestamp - attachment_timestamp < 11 * 60 * 1000
    }

    /// The message's addresses.
    pub fn addresses(&self) -> Vec<&IotaAddress> {
        match &self.payload {
            Payload::Transaction(tx) => tx
                .essence()
                .outputs()
                .iter()
                .map(|output| {
                    if let Output::SignatureLockedSingle(x) = output {
                        x.address()
                    } else {
                        unimplemented!()
                    }
                })
                .collect(),
            _ => vec![],
        }
    }

    /// Gets the absolute value of the transaction.
    pub fn compute_value(
        iota_message: &IotaMessage,
        id: &MessageId,
        account_addresses: &[Address],
    ) -> Value {
        let amount = match iota_message.payload().as_ref().unwrap() {
            Payload::Transaction(tx) => {
                let sent = !account_addresses
                    .iter()
                    .any(|address| address.outputs().iter().any(|o| o.message_id() == id));
                tx.essence().outputs().iter().fold(0, |acc, output| {
                    if let Output::SignatureLockedSingle(x) = output {
                        let address_belongs_to_account =
                            account_addresses.iter().any(|a| a.address() == x.address());
                        if sent {
                            if address_belongs_to_account {
                                acc
                            } else {
                                acc + x.amount().get()
                            }
                        } else if address_belongs_to_account {
                            acc + x.amount().get()
                        } else {
                            acc
                        }
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
#[derive(Debug, Clone, Deserialize_repr)]
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

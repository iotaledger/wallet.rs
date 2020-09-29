use crate::address::Address;
use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota::transaction::{
    prelude::{Hash, Message as IotaMessage, Payload},
    Vertex,
};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;

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

/// A transfer to make a transaction.
#[derive(Debug, Getters, Setters)]
#[getset(get = "pub")]
pub struct Transfer {
    /// The transfer value.
    amount: u64,
    /// The transfer address.
    address: Address,
    /// (Optional) transfer data.
    #[getset(set = "pub")]
    data: Option<String>,
}

impl Transfer {
    /// Initialises a new transfer to the given address.
    pub fn new(address: Address, amount: u64) -> Self {
        Self {
            address,
            amount,
            data: None,
        }
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
    value: i64,
    /// The value's unit.
    unit: ValueUnit,
}

impl Value {
    /// Ititialises a new Value.
    pub fn new(value: i64, unit: ValueUnit) -> Self {
        Self { value, unit }
    }

    /// Formats the value with its unit.
    pub fn with_denomination(&self) -> String {
        format!("{} {}", self.value, self.unit)
    }

    /// The transaction value without its unit.
    pub fn without_denomination(&self) -> i64 {
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
#[derive(
    Debug, Getters, Setters, Clone, Hash, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[getset(get = "pub", set = "pub(crate)")]
pub struct Message {
    /// The message version.
    pub(crate) version: u64,
    /// Message id of the first message this message refers to.
    pub(crate) trunk: Hash,
    /// Message id of the second message this message refers to.
    pub(crate) branch: Hash,
    /// Length of the payload.
    pub(crate) payload_length: u64,
    /// Transaction amount.
    pub(crate) payload: Payload,
    /// The transaction timestamp.
    pub(crate) timestamp: DateTime<Utc>,
    /// Transaction nonce.
    pub(crate) nonce: u64,
    /// Whether the transaction is confirmed or not.
    pub(crate) confirmed: bool,
    /// Whether the transaction is broadcasted or not.
    pub(crate) broadcasted: bool,
}

impl Message {
    pub(crate) fn from_iota_message(message: IotaMessage) -> crate::Result<Self> {
        let message = Self {
            version: 1,
            trunk: message.trunk().clone(),
            branch: message.branch().clone(),
            payload_length: 5, // TODO
            payload: todo!(),  // TODO message.payload,
            timestamp: Utc::now(),
            // TODO timestamp: DateTime::<Utc>::from_utc(
            //    NaiveDateTime::from_timestamp(*message.attachment_ts().to_inner() as i64, 0),
            //    Utc,
            // ),
            nonce: 5, // TODO message.nonce,
            confirmed: false,
            broadcasted: true,
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

    /// The message's address.
    pub fn address(&self) -> &Address {
        unimplemented!()
    }

    /// The message's hash.
    pub fn hash(&self) -> &Hash {
        unimplemented!()
    }

    /// Gets the absolute value of the transaction.
    pub fn value(&self) -> Value {
        let amount = match &self.payload {
            Payload::SignedTransaction(tx) => tx
                .unsigned_transaction
                .outputs
                .iter()
                .fold(0, |acc, output| acc + output.amount().get()),
            _ => 0,
        };
        Value::new(amount.try_into().unwrap(), ValueUnit::I)
    }
}

/// Message type.
#[derive(Debug, Clone, Deserialize)]
pub enum MessageType {
    /// Message received.
    Received,
    /// Message sent.
    Sent,
    /// Message not broadcasted.
    Failed,
    /// Message not confirmed.
    Unconfirmed,
    /// A value message.
    Value,
}

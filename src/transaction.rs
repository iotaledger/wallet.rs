use crate::address::{Address, AddressBuilder};
use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use getset::{Getters, Setters};
use iota::crypto::ternary::Hash;
use iota::transaction::{
    bundled::{BundledTransaction, BundledTransactionField, Tag as IotaTag},
    Vertex,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A transaction tag.
#[derive(Debug, Clone)]
pub struct Tag {
    tag: IotaTag,
}

impl Default for Tag {
    /// Initialises an empty tag.
    fn default() -> Self {
        Self {
            tag: IotaTag::zeros(),
        }
    }
}

impl Tag {
    /// Initialises a new tag.
    pub fn new(tag: IotaTag) -> Self {
        Self { tag }
    }

    /// Returns the tag formatted as ASCII.
    pub fn as_ascii(&self) -> String {
        let buf = self.tag.to_inner().encode::<T3B1Buf>();
        let trytes = buf.as_slice().as_trytes();
        trytes
            .iter()
            .map(|tryte| char::from(*tryte))
            .collect::<String>()
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
    /// The transfer transaction tag.
    #[getset(set = "pub")]
    tag: Option<IotaTag>,
    /// The transfer transaction message.
    #[getset(set = "pub")]
    message: Option<String>,
}

impl Transfer {
    /// Initialises a new transfer to the given address.
    pub fn new(address: Address, amount: u64) -> Self {
        Self {
            address,
            amount,
            tag: None,
            message: None,
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

/// A transaction definition.
#[derive(Debug, Getters, Setters, Clone)]
#[getset(get = "pub", set = "pub(crate)")]
pub struct Transaction {
    /// The transaction hash.
    pub(crate) hash: Hash,
    /// The transaction address.
    pub(crate) address: Address,
    /// The transaction amount.
    pub(crate) value: Value,
    /// The transaction tag.
    pub(crate) tag: Tag,
    /// The transaction timestamp.
    pub(crate) timestamp: DateTime<Utc>,
    /// The transaction current index in the bundle.
    pub(crate) current_index: u64,
    /// The transaction last index in the bundle.
    pub(crate) last_index: u64,
    /// The transaction bundle hash.
    pub(crate) bundle_hash: Hash,
    /// The trunk transaction hash.
    pub(crate) trunk_transaction: Hash,
    /// The branch transaction hash.
    pub(crate) branch_transaction: Hash,
    /// The transaction nonce.
    pub(crate) nonce: String,
    /// Whether the transaction is confirmed or not.
    pub(crate) confirmed: bool,
    /// Whether the transaction is broadcasted or not.
    pub(crate) broadcasted: bool,
}

impl Transaction {
    pub(crate) fn from_bundled(hash: Hash, tx: BundledTransaction) -> crate::Result<Self> {
        let transaction = Self {
            hash,
            address: AddressBuilder::new()
                .address(tx.address().clone())
                .key_index(0)
                .balance(0)
                .build()?,
            value: Value {
                value: *tx.value().to_inner(),
                unit: ValueUnit::I,
            },
            tag: Tag {
                tag: tx.tag().clone(),
            },
            timestamp: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(*tx.attachment_ts().to_inner() as i64, 0),
                Utc,
            ),
            current_index: *tx.index().to_inner() as u64,
            last_index: *tx.last_index().to_inner() as u64,
            trunk_transaction: *tx.trunk(),
            branch_transaction: *tx.branch(),
            bundle_hash: *tx.bundle(),
            nonce: "TX NONCE".to_string(), // TODO
            confirmed: false,
            broadcasted: true,
        };

        Ok(transaction)
    }

    /// Check if attachment timestamp on transaction is above max depth (~11 minutes)
    pub(crate) fn is_above_max_depth(&self) -> bool {
        let current_timestamp = Utc::now().timestamp();
        let attachment_timestamp = self.timestamp.timestamp();
        attachment_timestamp < current_timestamp
            && current_timestamp - attachment_timestamp < 11 * 60 * 1000
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

/// Transaction type.
#[derive(Debug, Clone, Deserialize)]
pub enum TransactionType {
    /// Transaction received.
    Received,
    /// Transaction sent.
    Sent,
    /// Transaction not broadcasted.
    Failed,
    /// Transaction not confirmed.
    Unconfirmed,
    /// A value transaction.
    Value,
}

use crate::address::{Address, AddressBuilder};
use bee_crypto::ternary::Hash;
use bee_transaction::{
  bundled::{BundledTransaction, BundledTransactionField, Tag as IotaTag},
  Vertex,
};
use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A transaction tag.
#[derive(Debug, Clone)]
pub struct Tag {
  tag: IotaTag,
}

impl Tag {
  /// Initialises a new tag.
  pub fn new(tag: IotaTag) -> Self {
    Self { tag }
  }

  /// Returns the tag as trytes.
  pub fn as_trytes(&self) -> &str {
    "trytes"
  }

  /// Returns the tag formatted as ASCII.
  pub fn as_ascii(&self) -> &str {
    "ascii"
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
    self.value
  }
}

/// A transaction definition.
#[derive(Getters, Setters, Clone)]
#[getset(get = "pub", set = "pub(crate)")]
pub struct Transaction {
  /// The transaction hash.
  hash: Hash,
  /// The transaction address.
  address: Address,
  /// The transaction amount.
  value: Value,
  /// The transaction tag.
  tag: Tag,
  /// The transaction timestamp.
  timestamp: DateTime<Utc>,
  /// The transaction current index in the bundle.
  current_index: u64,
  /// The transaction last index in the bundle.
  last_index: u64,
  /// The transaction bundle hash.
  bundle_hash: Hash,
  /// The trunk transaction hash.
  trunk_transaction: Hash,
  /// The branch transaction hash.
  branch_transaction: Hash,
  /// The transaction nonce.
  nonce: String,
  /// Whether the transaction is confirmed or not.
  confirmed: bool,
  /// Whether the transaction is broadcasted or not.
  broadcasted: bool,
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
      trunk_transaction: tx.trunk().clone(),
      branch_transaction: tx.branch().clone(),
      bundle_hash: tx.bundle().clone(),
      nonce: "TX NONCE".to_string(), // TODO
      confirmed: false,
      broadcasted: true,
    };

    Ok(transaction)
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

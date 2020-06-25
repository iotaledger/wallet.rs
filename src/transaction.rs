use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota_bundle_preview::{Address as IotaAddress, Hash, Tag as IotaTag};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A transaction tag.
#[derive(Serialize, Deserialize)]
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
#[derive(Getters, Setters)]
pub struct Transfer<'a> {
  /// The transfer value.
  #[getset(get = "pub")]
  amount: f64,
  /// The transfer address.
  #[getset(get = "pub")]
  address: IotaAddress,
  /// The transfer transaction tag.
  #[getset(get = "pub", set = "pub")]
  tag: Option<IotaTag>,
  /// The transfer transaction message.
  #[getset(get = "pub", set = "pub")]
  message: Option<&'a str>,
}

impl<'a> Transfer<'a> {
  /// Initialises a new transfer to the given address.
  pub fn new(address: IotaAddress, amount: f64) -> Self {
    Self {
      address,
      amount,
      tag: None,
      message: None,
    }
  }
}

/// Possible Value units.
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct Value {
  /// The value.
  value: f64,
  /// The value's unit.
  unit: ValueUnit,
}

impl Value {
  /// Ititialises a new Value.
  pub fn new(value: f64, unit: ValueUnit) -> Self {
    Self { value, unit }
  }

  /// Formats the value with its unit.
  pub fn with_denomination(&self) -> String {
    format!("{} {}", self.value, self.unit)
  }

  /// The transaction value without its unit.
  pub fn without_denomination(&self) -> f64 {
    self.value
  }
}

// TODO Hash serde
/// A transaction definition.
#[derive(Getters, Serialize, Deserialize)]
pub struct Transaction<'a> {
  /*/// The transaction hash.
  #[getset(get = "pub")]
  hash: Hash,*/
  /// The transaction address.
  #[getset(get = "pub")]
  address: IotaAddress,
  /// The transaction amount.
  #[getset(get = "pub")]
  value: Value,
  /// The transaction tag.
  #[getset(get = "pub")]
  tag: Tag,
  /// The transaction timestamp.
  #[getset(get = "pub")]
  timestamp: DateTime<Utc>,
  /// The transaction current index in the bundle.
  #[getset(get = "pub")]
  current_index: u64,
  /// The transaction last index in the bundle.
  #[getset(get = "pub")]
  last_index: u64,
  /*/// The transaction bundle hash.
  #[getset(get = "pub")]
  bundle_hash: Hash,
  /// The trunk transaction hash.
  #[getset(get = "pub")]
  trunk_transaction: Hash,
  /// The branch transaction hash.
  #[getset(get = "pub")]
  brach_transaction: Hash,*/
  /// The transaction nonce.
  #[getset(get = "pub")]
  nonce: &'a str,
  /// Whether the transaction is confirmed or not.
  #[getset(get = "pub")]
  confirmed: bool,
  /// Whether the transaction is broadcasted or not.
  #[getset(get = "pub")]
  broadcasted: bool,
}

use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use iota_bundle_preview::{Address as IotaAddress, Hash, Tag as IotaTag, TransactionField};
use iota_ternary_preview::{T3B1Buf, Tryte, TryteBuf};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryInto;
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
  amount: u64,
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
  pub fn new(address: IotaAddress, amount: u64) -> Self {
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
    self.value
  }
}

struct HashStringVisitor;

impl<'de> Visitor<'de> for HashStringVisitor {
  type Value = HashDef;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a Hash string")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let tryte_buf = TryteBuf::new();
    for character in value.chars() {
      let tryte: Tryte = character
        .try_into()
        .expect("failed to convert char to Tryte");
    }

    let mut trits = [0; 243];
    trits.copy_from_slice(tryte_buf.as_trits().encode::<T3B1Buf>().as_i8_slice());
    Ok(HashDef(trits))
  }
}

/// Hash wrapper to facilitate serialize/deserialize operations.
pub struct HashDef([i8; 243]);

impl Serialize for HashDef {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut chars: Vec<char> = Vec::new();
    for value in self.0.iter() {
      let tryte: Tryte = (*value).try_into().expect("failed to convert to Tryte");
      chars.push(tryte.into());
    }
    Vec::serialize(&chars, serializer)
  }
}

impl<'de> Deserialize<'de> for HashDef {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_string(HashStringVisitor {})
  }
}

impl From<&HashDef> for Hash {
  fn from(def: &HashDef) -> Hash {
    let mut tryte_buf = TryteBuf::new();
    for value in def.0.iter() {
      let tryte: Tryte = (*value).try_into().expect("failed to convert to Tryte");
      tryte_buf.push(tryte);
    }
    Hash::from_inner_unchecked(tryte_buf.as_trits().encode())
  }
}

/// A transaction definition.
#[derive(Getters, Serialize, Deserialize)]
pub struct Transaction<'a> {
  /// The transaction hash.
  hash: HashDef,
  /// The transaction address.
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
  /// The transaction bundle hash.
  bundle_hash: HashDef,
  /// The trunk transaction hash.
  trunk_transaction: HashDef,
  /// The branch transaction hash.
  brach_transaction: HashDef,
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

impl<'a> Transaction<'a> {
  /// The transaction hash.
  pub fn hash(&self) -> Hash {
    (&self.hash).into()
  }
}

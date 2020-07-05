use crate::address::Address;
use bee_crypto::ternary::Hash;
use bee_ternary::{T1B1Buf, TritBuf, Trits, Tryte, TryteBuf, T1B1};
use bee_transaction::bundled::{BundledTransactionField, Tag as IotaTag};
use chrono::prelude::{DateTime, Utc};
use getset::{Getters, Setters};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryInto;
use std::fmt;

/// A transaction tag.
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
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

/// Hash wrapper to facilitate serialize/deserialize operations.
#[derive(Clone)]
struct HashDef([i8; 243]);

impl PartialEq for HashDef {
  fn eq(&self, other: &HashDef) -> bool {
    self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
  }
}

impl Serialize for HashDef {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    TritBuf::serialize(
      &Trits::<T1B1>::try_from_raw(&self.0, 243)
        .map_err(|_| SerError::custom("failed to get Trits from Hash"))?
        .to_buf::<T1B1Buf>(),
      serializer,
    )
  }
}

impl<'de> Deserialize<'de> for HashDef {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    TritBuf::deserialize(deserializer).map(|buf: TritBuf<T1B1Buf>| {
      let mut trits = [0; 243];
      trits.copy_from_slice(buf.as_slice().encode::<T1B1Buf>().as_i8_slice());
      HashDef(trits)
    })
  }
}

// TODO this seems wrong
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
#[derive(Getters, Serialize, Deserialize, Clone)]
pub struct Transaction {
  /// The transaction hash.
  hash: HashDef,
  /// The transaction address.
  address: Address,
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
  nonce: String,
  /// Whether the transaction is confirmed or not.
  #[getset(get = "pub")]
  confirmed: bool,
  /// Whether the transaction is broadcasted or not.
  #[getset(get = "pub")]
  broadcasted: bool,
}

impl PartialEq for Transaction {
  fn eq(&self, other: &Self) -> bool {
    self.hash() == other.hash()
  }
}

impl Transaction {
  /// The transaction hash.
  pub fn hash(&self) -> Hash {
    (&self.hash).into()
  }
}

#[cfg(test)]
mod tests {
  use super::HashDef;
  use bee_ternary::{T1B1Buf, TryteBuf};

  #[test]
  fn serde_hash() {
    let tryte_buf = TryteBuf::try_from_str(
      "RVORZ9SIIP9RCYMREUIXXVPQIPHVCNPQ9HZWYKFWYWZRE9JQKG9REPKIASHUUECPSQO9JT9XNMVKWYGVA",
    )
    .unwrap();

    let mut trits = [0; 243];
    trits.copy_from_slice(tryte_buf.as_trits().encode::<T1B1Buf>().as_i8_slice());
    let hash = HashDef(trits);

    let serialized = serde_json::to_string(&hash).expect("failed to serialize hash");
    let deserialized: HashDef =
      serde_json::from_str(&serialized).expect("failed to deserialize hash");

    assert!(hash == deserialized);
  }
}

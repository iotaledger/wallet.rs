pub use bee_transaction::bundled::Address as IotaAddress;
use getset::Getters;
use serde::{Deserialize, Serialize};

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
  address: Option<IotaAddress>,
  balance: Option<u64>,
  key_index: Option<u64>,
  // TODO checksum:
}

impl AddressBuilder {
  /// Initialises a new instance of the address builder.
  pub fn new() -> AddressBuilder {
    Default::default()
  }

  /// Defines the address.
  pub fn address(mut self, address: IotaAddress) -> Self {
    self.address = Some(address);
    self
  }

  /// Sets the address balance.
  pub fn balance(mut self, balance: u64) -> Self {
    self.balance = Some(balance);
    self
  }

  /// Sets the address key index.
  pub fn key_index(mut self, key_index: u64) -> Self {
    self.key_index = Some(key_index);
    self
  }

  /// Builds the address.
  pub fn build(self) -> crate::Result<Address> {
    let address = Address {
      address: self
        .address
        .ok_or_else(|| anyhow::anyhow!("the `address` field is required"))?,
      balance: self
        .balance
        .ok_or_else(|| anyhow::anyhow!("the `balance` field is required"))?,
      key_index: self
        .key_index
        .ok_or_else(|| anyhow::anyhow!("the `key_index` field is required"))?,
    };
    Ok(address)
  }
}

/// An address.
#[derive(Getters, Serialize, Deserialize)]
pub struct Address {
  /// The address.
  #[getset(get = "pub")]
  address: IotaAddress,
  /// The address balance.
  #[getset(get = "pub")]
  balance: u64,
  /// The address key index.
  #[getset(get = "pub")]
  key_index: u64,
}

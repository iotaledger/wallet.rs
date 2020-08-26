use crate::account::Account;
use bee_crypto::ternary::sponge::{Kerl, Sponge};
use bee_signing::ternary::{
    wots::{WotsSecurityLevel, WotsShakePrivateKeyGeneratorBuilder},
    PrivateKey, PrivateKeyGenerator, PublicKey,
};
use bee_ternary::TritBuf;
pub use bee_transaction::bundled::Address as IotaAddress;
use bee_transaction::bundled::BundledTransactionField;
use getset::Getters;

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
    address: Option<IotaAddress>,
    balance: Option<u64>,
    key_index: Option<u64>,
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
        let iota_address = self
            .address
            .ok_or_else(|| anyhow::anyhow!("the `address` field is required"))?;
        let checksum = generate_checksum(&iota_address)?;
        let address = Address {
            address: iota_address,
            balance: self
                .balance
                .ok_or_else(|| anyhow::anyhow!("the `balance` field is required"))?,
            key_index: self
                .key_index
                .ok_or_else(|| anyhow::anyhow!("the `key_index` field is required"))?,
            checksum,
        };
        Ok(address)
    }
}

/// An address.
#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    address: IotaAddress,
    /// The address balance.
    balance: u64,
    /// The address key index.
    key_index: u64,
    /// The address checksum.
    checksum: TritBuf,
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.key_index() == other.key_index()
    }
}

/// Gets an unused address for the given account.
pub(crate) async fn get_new_address(account: &Account) -> crate::Result<Address> {
  let client = crate::client::get_client(account.client_options());
  let (key_index, iota_address) = client
    .generate_new_address(account.seed())
    .generate()
    .await?;
  let balance = get_balance(&account, &iota_address).await?;
  let checksum = generate_checksum(&iota_address)?;
  let address = Address {
    address: iota_address,
    balance,
    key_index,
    checksum,
  };
  Ok(address)
}

/// Batch address generation.
pub(crate) async fn get_addresses(account: &Account, count: u64) -> crate::Result<Vec<Address>> {
  let mut addresses = vec![];
  let seed_trits = account.seed().as_trits();
  for i in 0..count {
    let address: IotaAddress = IotaAddress::try_from_inner(
      WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
        .with_security_level(WotsSecurityLevel::Medium)
        .build()
        .unwrap()
        .generate_from_entropy(seed_trits)
        .unwrap()
        .generate_public_key()
        .unwrap()
        .as_trits()
        .to_owned(),
    )
    .unwrap();
    let balance = get_balance(&account, &address).await?;
    let checksum = generate_checksum(&address)?;
    addresses.push(Address {
      address,
      balance,
      key_index: i, // TODO
      checksum,
    })
  }
  Ok(addresses)
}

/// Generates a checksum for the given address
// TODO: maybe this should be part of the crypto lib
pub(crate) fn generate_checksum(address: &IotaAddress) -> crate::Result<TritBuf> {
  let mut kerl = Kerl::new();
  let mut hash = kerl
    .digest(address.to_inner())
    .map_err(|e| anyhow::anyhow!("Erro hashing the address"))?;
  let mut trits = vec![];

    for _ in 1..10 {
        if let Some(trit) = hash.pop() {
            trits.push(trit);
        } else {
            return Err(anyhow::anyhow!("Hash error"));
        }
    }

    Ok(TritBuf::from_trits(&trits[..]))
}

async fn get_balance(account: &Account, address: &IotaAddress) -> crate::Result<u64> {
  let client = crate::client::get_client(account.client_options());
  client
    .get_balances()
    .addresses(&[address.clone()])
    .send()
    .await?
    .balances
    .first()
    .map(|v| *v)
    .ok_or_else(|| anyhow::anyhow!("Balances response empty"))
}

pub(crate) fn is_unspent(account: &Account, address: &IotaAddress) -> bool {
  account
    .transactions()
    .iter()
    .any(|tx| tx.value().without_denomination() < 0 && tx.address().address() == address)
}

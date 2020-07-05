use bee_crypto::ternary::Kerl;
use bee_signing::ternary::{
  PrivateKey, PrivateKeyGenerator, PublicKey, Seed, TernarySeed, WotsSecurityLevel,
  WotsShakePrivateKeyGeneratorBuilder,
};
use bee_transaction::bundled::{Address, BundledTransactionField};

pub struct GenerateAddressBuilder<'a> {
  seed: Option<&'a TernarySeed<Kerl>>,
}

impl<'a> GenerateAddressBuilder<'a> {
  pub(crate) fn new() -> Self {
    Self { seed: None }
  }

  pub fn seed(mut self, seed: &'a TernarySeed<Kerl>) -> Self {
    self.seed = Some(seed);
    self
  }

  pub fn generate(self) -> crate::Result<Address> {
    let seed = self
      .seed
      .ok_or_else(|| anyhow::anyhow!("seed is required"))?;

    let address: Address = Address::try_from_inner(
      WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
        .security_level(WotsSecurityLevel::Medium)
        .build()
        .unwrap()
        .generate_from_entropy(&seed.trits())
        .unwrap()
        .generate_public_key()
        .unwrap()
        .trits()
        .to_owned(),
    )
    .unwrap();

    Ok(address)
  }
}

#[cfg(test)]
mod tests {
  use super::GenerateAddressBuilder;
  use bee_crypto::ternary::Kerl;
  use bee_signing::ternary::{Seed, TernarySeed};
  use bee_ternary::{T1B1Buf, TryteBuf};

  fn generate_address_fails_without_seed() {
    let result = GenerateAddressBuilder::new().generate();
    assert!(result.is_err());
  }

  fn generate_address_check() {
    let seed = TernarySeed::<Kerl>::from_buf(
      TryteBuf::try_from_str(
        "RVORZ9SIIP9RCYMREUIXXVPQIPHVCNPQ9HZWYKFWYWZRE9JQKG9REPKIASHUUECPSQO9JT9XNMVKWYGVA",
      )
      .unwrap()
      .as_trits()
      .encode::<T1B1Buf>(),
    )
    .unwrap();

    let address = GenerateAddressBuilder::new()
      .seed(&seed)
      .generate()
      .unwrap();
  }
}

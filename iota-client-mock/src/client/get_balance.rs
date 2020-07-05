use bee_crypto::ternary::Kerl;
use bee_signing::ternary::TernarySeed;
use bee_transaction::bundled::Address;

pub struct GetBalanceBuilder<'a> {
  seed: Option<&'a TernarySeed<Kerl>>,
}

impl<'a> GetBalanceBuilder<'a> {
  pub(crate) fn new() -> Self {
    Self { seed: None }
  }

  pub fn seed(mut self, seed: &'a TernarySeed<Kerl>) -> Self {
    self.seed = Some(seed);
    self
  }

  pub fn get() -> crate::Result<u64> {
    Ok(5)
  }
}

pub struct GetBalanceForAddressBuilder {
  address: Address,
}

impl GetBalanceForAddressBuilder {
  pub(crate) fn new(address: Address) -> Self {
    Self { address }
  }

  pub fn get() -> crate::Result<u64> {
    Ok(17)
  }
}

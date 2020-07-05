use super::Converter;
use bee_crypto::ternary::Hash;
use bee_crypto::ternary::Kerl;
use bee_signing::ternary::TernarySeed;
use bee_ternary::TryteBuf;
use bee_transaction::bundled::{Address, BundledTransactionField, Tag};

pub struct SendBuilder<'a> {
  address: Address,
  value: Option<u64>,
  seed: Option<&'a TernarySeed<Kerl>>,
  message: Option<String>,
  tag: Option<Tag>,
  converter: Converter,
}

impl<'a> SendBuilder<'a> {
  pub(crate) fn new(address: Address) -> Self {
    Self {
      address,
      value: None,
      seed: None,
      message: None,
      tag: None,
      converter: Converter::UTF8,
    }
  }

  pub fn value(mut self, value: u64) -> Self {
    self.value = Some(value);
    self
  }

  pub fn seed(mut self, seed: &'a TernarySeed<Kerl>) -> Self {
    self.seed = Some(seed);
    self
  }

  pub fn message(mut self, message: String) -> Self {
    self.message = Some(message);
    self
  }

  pub fn tag(mut self, tag: Tag) -> Self {
    self.tag = Some(tag);
    self
  }

  pub fn converter(mut self, converter: Converter) -> Self {
    self.converter = converter;
    self
  }

  pub fn send(self) -> crate::Result<Hash> {
    let hash = Hash::from_inner_unchecked(
      TryteBuf::try_from_str(
        "KXRVLFETGUTUWBCNCC9DWO99JQTEI9YXVOZHWELSYP9SG9KN9WCKXOVTEFHFH9EFZJKFYCZKQPPBXYSGJ",
      )
      .unwrap()
      .as_trits()
      .encode(),
    );
    Ok(hash)
  }
}

#[cfg(test)]
mod tests {
  use super::SendBuilder;
  use bee_transaction::bundled::Address;

  #[test]
  fn send_returns_a_hash() {
    let hash = SendBuilder::new(Address::zeros())
      .send()
      .expect("failed to send");
  }
}

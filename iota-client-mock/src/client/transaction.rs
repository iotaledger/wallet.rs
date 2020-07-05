use super::Converter;
use bee_crypto::ternary::Hash;
use bee_transaction::bundled::{Address, BundledTransactionField, Tag, Timestamp};

pub struct Transaction {
  hash: Hash,
  address: Address,
  tag: Option<Tag>,
  created_at: Timestamp,
  attached_at: Timestamp,
}

pub struct GetTransactionBuilder {
  transaction_hash: Hash,
  converter: Converter,
}

impl GetTransactionBuilder {
  pub(crate) fn new(transaction_hash: Hash) -> Self {
    Self {
      transaction_hash,
      converter: Converter::UTF8,
    }
  }

  pub fn converter(mut self, converter: Converter) -> Self {
    self.converter = converter;
    self
  }

  pub async fn get() -> crate::Result<Transaction> {
    let transaction = Transaction {
      address: Address::zeros(),
      hash: Hash::zeros(),
      tag: None,
      created_at: Timestamp::from_inner_unchecked(0),
      attached_at: Timestamp::from_inner_unchecked(0),
    };
    Ok(transaction)
  }
}

pub struct FindTransactionsBuilder {
  transaction_hashes: Option<Vec<Hash>>,
  address: Option<Address>,
  tag: Option<Tag>,
  tag_prefix: Option<String>,
  offset: u64,
  limit: u64,
  converter: Converter,
}

impl FindTransactionsBuilder {
  pub(crate) fn new() -> Self {
    Self {
      transaction_hashes: None,
      address: None,
      tag: None,
      tag_prefix: None,
      offset: 0,
      limit: 100,
      converter: Converter::UTF8,
    }
  }

  pub fn transaction_hashes(mut self, transaction_hashes: Vec<Hash>) -> Self {
    self.transaction_hashes = Some(transaction_hashes);
    self
  }

  pub fn address(mut self, address: Address) -> Self {
    self.address = Some(address);
    self
  }

  pub fn tag(mut self, tag: Tag) -> Self {
    self.tag = Some(tag);
    self
  }

  pub fn tag_prefix(mut self, tag_prefix: String) -> Self {
    self.tag_prefix = Some(tag_prefix);
    self
  }

  pub fn offset(mut self, offset: u64) -> Self {
    self.offset = offset;
    self
  }

  pub fn limit(mut self, limit: u64) -> Self {
    self.limit = limit;
    self
  }

  pub fn converter(mut self, converter: Converter) -> Self {
    self.converter = converter;
    self
  }

  pub async fn get(self) -> crate::Result<Vec<Transaction>> {
    let transactions = vec![Transaction {
      address: self.address.unwrap_or(Address::zeros()),
      hash: self
        .transaction_hashes
        .map(|h| h[0])
        .unwrap_or(Hash::zeros()),
      tag: self.tag,
      created_at: Timestamp::from_inner_unchecked(0),
      attached_at: Timestamp::from_inner_unchecked(0),
    }];
    Ok(transactions)
  }
}

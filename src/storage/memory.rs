use super::StorageAdapter;
use crate::account::Account;
use std::collections::HashMap;

/// In memory storage adapter.
/// Useful for simple tests.
pub struct MemoryStorageAdapter<'a> {
  collection: HashMap<String, Account<'a>>,
}

impl<'a> MemoryStorageAdapter<'a> {
  /// Initialises the memory storage adapter.
  pub fn new() -> Self {
    Self {
      collection: HashMap::new(),
    }
  }
}

impl<'a> StorageAdapter<'a> for MemoryStorageAdapter<'a> {
  fn get(&mut self, account_id: &str) -> std::result::Result<&Account<'a>, anyhow::Error> {
    self
      .collection
      .get(account_id)
      .ok_or(anyhow::anyhow!("Account not found"))
  }

  fn get_all(&mut self) -> std::result::Result<std::vec::Vec<&Account<'a>>, anyhow::Error> {
    let accounts = self.collection.iter().map(|(_, v)| v).collect();
    Ok(accounts)
  }

  fn set(
    &mut self,
    account_id: &str,
    account: Account<'a>,
  ) -> std::result::Result<(), anyhow::Error> {
    self.collection.insert(account_id.to_string(), account);
    Ok(())
  }

  fn remove(&mut self, account_id: &str) -> std::result::Result<(), anyhow::Error> {
    self.collection.remove(account_id);
    Ok(())
  }
}

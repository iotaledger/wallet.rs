use super::StorageAdapter;
use std::collections::HashMap;

/// In memory storage adapter.
/// Useful for simple tests.
#[derive(Default, Clone)]
pub struct MemoryStorageAdapter {
  collection: HashMap<String, String>,
}

impl MemoryStorageAdapter {
  /// Initialises the memory storage adapter.
  pub fn new() -> Self {
    Self {
      collection: HashMap::new(),
    }
  }
}

impl StorageAdapter for MemoryStorageAdapter {
  fn get(&mut self, account_id: &str) -> crate::Result<&String> {
    self
      .collection
      .get(account_id)
      .ok_or_else(|| anyhow::anyhow!("Account not found"))
  }

  fn get_all(&mut self) -> crate::Result<std::vec::Vec<&String>> {
    let accounts = self.collection.iter().map(|(_, v)| v).collect();
    Ok(accounts)
  }

  fn set(&mut self, account_id: &str, account: String) -> std::result::Result<(), anyhow::Error> {
    self.collection.insert(account_id.to_string(), account);
    Ok(())
  }

  fn remove(&mut self, account_id: &str) -> std::result::Result<(), anyhow::Error> {
    self.collection.remove(account_id);
    Ok(())
  }
}

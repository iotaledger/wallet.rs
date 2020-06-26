use super::StorageAdapter;
use kv::*;
use std::path::Path;

/// Key value storage adapter.
pub struct KeyValueStorageAdapter<'a> {
  bucket: Bucket<'a, String, String>,
}

impl<'a> KeyValueStorageAdapter<'a> {
  /// Initialises the storage adapter.
  pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    // Configure the database
    let cfg = Config::new(path);

    // Open the key/value store
    let store = Store::new(cfg)?;

    // A Bucket provides typed access to a section of the key/value store
    let account_bucket = store.bucket::<String, String>(Some("accounts"))?;

    let adapter = Self {
      bucket: account_bucket,
    };
    Ok(adapter)
  }
}

impl<'a> StorageAdapter for KeyValueStorageAdapter<'a> {
  fn get(&self, account_id: &str) -> crate::Result<String> {
    let account = self.bucket.get(account_id)?;
    account.ok_or_else(|| anyhow::anyhow!("account isn't stored"))
  }

  fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
    let accounts = self
      .bucket
      .iter()
      .map(|item| item.unwrap().value().unwrap())
      .collect();
    Ok(accounts)
  }

  fn set(&self, account_id: &str, account: String) -> std::result::Result<(), anyhow::Error> {
    self.bucket.set(account_id, account)?;
    Ok(())
  }

  fn remove(&self, account_id: &str) -> std::result::Result<(), anyhow::Error> {
    self.bucket.remove(account_id)?;
    Ok(())
  }
}

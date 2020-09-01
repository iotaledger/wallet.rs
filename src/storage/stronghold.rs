use super::sqlite::SqliteStorageAdapter;
use super::StorageAdapter;
use crate::account::AccountIdentifier;
use std::path::Path;

use stronghold::{Base64Decodable, Id as StrongholdId, Stronghold};

/// Stronghold storage adapter.
pub struct StrongholdStorageAdapter {
    id_storage: SqliteStorageAdapter,
    stronghold: Stronghold,
}

impl StrongholdStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        std::fs::create_dir_all(&path)?;
        let id_storage = SqliteStorageAdapter::new(path.as_ref().join("id.db"), "account_ids")?;
        let storage_path = path.as_ref().join("snapshot");
        let adapter = Self {
            id_storage,
            stronghold: Stronghold::new(storage_path),
        };
        Ok(adapter)
    }
}

fn create_stronghold_id(id: String) -> crate::Result<StrongholdId> {
    let bytes = Vec::from_base64(id.as_bytes())?;
    let id = StrongholdId::load(&bytes)?;
    Ok(id)
}

impl StorageAdapter for StrongholdStorageAdapter {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        let stronghold_id_string = self.id_storage.get(account_id)?;
        let stronghold_id = create_stronghold_id(stronghold_id_string)?;
        let account = self.stronghold.record_read(&stronghold_id, "password");
        Ok(account)
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let ids = self.id_storage.get_all()?;
        for id in ids {
            let id = create_stronghold_id(id)?;
            let account = self.stronghold.record_read(&id, "password");
            accounts.push(account);
        }
        Ok(accounts)
    }

    fn set(
        &self,
        account_id: AccountIdentifier,
        account: String,
    ) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id = self
            .stronghold
            .record_create("", account.as_str(), "password");
        self.id_storage
            .set(account_id, format!("{:?}", stronghold_id))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id_string = self.id_storage.get(account_id.clone())?;
        let stronghold_id = create_stronghold_id(stronghold_id_string)?;
        self.stronghold.record_remove(stronghold_id, "password");
        self.id_storage.remove(account_id)?;
        Ok(())
    }
}

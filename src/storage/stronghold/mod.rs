mod interface;

use super::sqlite::SqliteStorageAdapter;
use super::StorageAdapter;
use crate::account::AccountIdentifier;
use std::path::{Path, PathBuf};

/// Stronghold storage adapter.
pub struct StrongholdStorageAdapter {
    storage_path: PathBuf,
    id_storage: SqliteStorageAdapter,
}

impl StrongholdStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        std::fs::create_dir_all(&path)?;
        let id_storage = SqliteStorageAdapter::new(path.as_ref().join("id.db"), "account_ids")?;
        let storage_path = path.as_ref().join("snapshot");
        let adapter = Self {
            id_storage,
            storage_path,
        };
        Ok(adapter)
    }
}

impl StorageAdapter for StrongholdStorageAdapter {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        let stronghold_id = self.id_storage.get(account_id)?;
        let account = interface::read(&self.storage_path, "password", stronghold_id);
        Ok(account.expect("failed to read account"))
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let ids = interface::list_ids(&self.storage_path, "password");
        for id in ids {
            let account = interface::read(&self.storage_path, "password", id);
            accounts.push(account.expect("failed to read account"));
        }
        Ok(accounts)
    }

    fn set(
        &self,
        account_id: AccountIdentifier,
        account: String,
    ) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id = interface::encrypt(&self.storage_path, "password", &account);
        self.id_storage
            .set(account_id, format!("{:?}", stronghold_id))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id = self.id_storage.get(account_id)?;
        interface::revoke(&self.storage_path, "password", stronghold_id);
        Ok(())
    }
}

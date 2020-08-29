mod interface;

use super::sqlite::SqliteStorageAdapter;
use super::StorageAdapter;
use crate::account::AccountIdentifier;
use std::path::Path;

/// Stronghold storage adapter.
pub struct StrongholdStorageAdapter {
    id_storage: SqliteStorageAdapter,
}

impl StrongholdStorageAdapter {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let id_storage = SqliteStorageAdapter::new(path, "account_ids")?;
        let adapter = Self { id_storage };
        Ok(adapter)
    }
}

impl StorageAdapter for StrongholdStorageAdapter {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        let stronghold_id = self.id_storage.get(account_id)?;
        let account = interface::read("password", stronghold_id);
        Ok(account.expect("failed to read account"))
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let ids: Vec<String> = self.id_storage.get_all()?;
        for id in ids {
            let account = interface::read("password", id);
            accounts.push(account.expect("failed to read account"));
        }
        Ok(accounts)
    }

    fn set(
        &self,
        account_id: AccountIdentifier,
        account: String,
    ) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id = interface::encrypt("password", &account);
        self.id_storage
            .set(account_id, format!("{:?}", stronghold_id))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> std::result::Result<(), anyhow::Error> {
        let stronghold_id = self.id_storage.get(account_id)?;
        interface::revoke("password", stronghold_id);
        Ok(())
    }
}

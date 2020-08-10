mod interface;

use super::StorageAdapter;
use crate::account::AccountIdentifier;
use kv::*;
use std::path::Path;

/// Key value storage adapter.
pub struct StrongholdStorageAdapter<'a> {
    id_bucket: Bucket<'a, String, String>,
}

impl<'a> StrongholdStorageAdapter<'a> {
    /// Initialises the storage adapter.
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        // Configure the database
        let cfg = Config::new(path);

        // Open the key/value store
        let store = Store::new(cfg)?;

        // A Bucket provides typed access to a section of the key/value store
        let id_bucket = store.bucket::<String, String>(Some("ids"))?;

        let adapter = Self { id_bucket };
        Ok(adapter)
    }
}

impl<'a> StorageAdapter for StrongholdStorageAdapter<'a> {
    fn get(&self, account_id: AccountIdentifier) -> crate::Result<String> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let stronghold_id = self.id_bucket.get(id)?.unwrap();
        let account = interface::read("password", stronghold_id);
        Ok(account.expect("failed to read account"))
    }

    fn get_all(&self) -> crate::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let ids: Vec<String> = self
            .id_bucket
            .iter()
            .map(|item| item.unwrap().value().unwrap())
            .collect();
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
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let stronghold_id = interface::encrypt("password", &account);
        self.id_bucket.set(id, format!("{:?}", stronghold_id))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> std::result::Result<(), anyhow::Error> {
        let id = match account_id {
            AccountIdentifier::Id(id) => id,
            _ => return Err(anyhow::anyhow!("only Id is supported")),
        };
        let stronghold_id = self.id_bucket.get(id)?.unwrap();
        interface::revoke("password", stronghold_id);
        Ok(())
    }
}

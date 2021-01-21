// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// ! An example of using a custom storage adapter (in this case, using sled).
use iota_wallet::{
    account_manager::{AccountManager, ManagerStorage},
    client::ClientOptionsBuilder,
    signing::SignerType,
    storage::StorageAdapter,
};

struct MyStorage {
    db: sled::Db,
}

impl MyStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let instance = Self { db: sled::open(path)? };
        Ok(instance)
    }
}

#[async_trait::async_trait]
impl StorageAdapter for MyStorage {
    async fn get(&mut self, account_id: &str) -> iota_wallet::Result<String> {
        match self.db.get(account_id) {
            Ok(Some(value)) => Ok(String::from_utf8(value.to_vec()).unwrap()),
            Ok(None) => Err(iota_wallet::Error::AccountNotFound),
            Err(e) => Err(iota_wallet::Error::Storage(format!(
                "operational problem encountered: {}",
                e
            ))),
        }
    }

    async fn get_all(&mut self) -> iota_wallet::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        for tuple in self.db.iter() {
            let (_, value) = tuple.unwrap();
            accounts.push(String::from_utf8(value.to_vec()).unwrap());
        }
        Ok(accounts)
    }

    async fn set(&mut self, account_id: &str, account: String) -> iota_wallet::Result<()> {
        self.db
            .insert(account_id, account.as_bytes())
            .map_err(|e| iota_wallet::Error::Storage(e.to_string()))?;
        Ok(())
    }

    async fn remove(&mut self, account_id: &str) -> iota_wallet::Result<()> {
        self.db
            .remove(account_id)
            .map_err(|e| iota_wallet::Error::Storage(e.to_string()))?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::builder()
        .with_storage(
            "./test-storage/sled",
            ManagerStorage::Custom(Box::new(
                MyStorage::new("./test-storage/sled").map_err(|e| iota_wallet::Error::Storage(e.to_string()))?,
            )),
            None,
        )?
        .finish()
        .await
        .unwrap();
    manager.set_stronghold_password("password").await.unwrap();
    manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

    // first we'll create an example account
    let client_options = ClientOptionsBuilder::node("https://api.lb-0.testnet.chrysalis2.com")?.build();
    manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    Ok(())
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// ! An example of using a custom storage adapter (in this case, using sled).

#[cfg(not(any(feature = "sqlite-storage", feature = "stronghold-storage")))]
use iota_wallet::{
    account::AccountIdentifier, account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType,
    storage::StorageAdapter,
};

#[cfg(any(feature = "sqlite-storage", feature = "stronghold-storage"))]
fn main() {
    panic!("can't run custom adapter example with `sqlite-storage` or `stronghold-storage` features; use `--no-default-features` to run")
}

#[cfg(not(any(feature = "sqlite-storage", feature = "stronghold-storage")))]
#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    struct MyStorage {
        db: sled::Db,
    }

    impl MyStorage {
        pub fn new<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
            let instance = Self { db: sled::open(path)? };
            Ok(instance)
        }
    }

    fn account_id_value(account_id: &AccountIdentifier) -> iota_wallet::Result<String> {
        match account_id {
            AccountIdentifier::Id(val) => Ok(val.to_string()),
            _ => Err(iota_wallet::Error::Storage(
                "Unexpected AccountIdentifier type".to_string(),
            )),
        }
    }

    #[async_trait::async_trait]
    impl StorageAdapter for MyStorage {
        async fn get(&self, account_id: &AccountIdentifier) -> iota_wallet::Result<String> {
            match self.db.get(account_id_value(account_id)?) {
                Ok(Some(value)) => Ok(String::from_utf8(value.to_vec()).unwrap()),
                Ok(None) => Err(iota_wallet::Error::AccountNotFound),
                Err(e) => Err(iota_wallet::Error::Storage(format!(
                    "operational problem encountered: {}",
                    e
                ))),
            }
        }

        async fn get_all(&self) -> iota_wallet::Result<std::vec::Vec<String>> {
            let mut accounts = vec![];
            for tuple in self.db.iter() {
                let (_, value) = tuple.unwrap();
                accounts.push(String::from_utf8(value.to_vec()).unwrap());
            }
            Ok(accounts)
        }

        async fn set(&self, account_id: &AccountIdentifier, account: String) -> iota_wallet::Result<()> {
            self.db
                .insert(account_id_value(account_id)?, account.as_bytes())
                .map_err(|e| iota_wallet::Error::Storage(e.to_string()))?;
            Ok(())
        }

        async fn remove(&self, account_id: &AccountIdentifier) -> iota_wallet::Result<()> {
            self.db
                .remove(account_id_value(account_id)?)
                .map_err(|e| iota_wallet::Error::Storage(e.to_string()))?;
            Ok(())
        }
    }

    let mut manager = AccountManager::builder()
        .with_storage(
            "./test-storage/sled",
            MyStorage::new("./test-storage/sled").map_err(|e| iota_wallet::Error::Storage(e.to_string()))?,
        )
        .finish()
        .await
        .unwrap();
    manager.set_stronghold_password("password").await.unwrap();
    manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

    // first we'll create an example account
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    Ok(())
}

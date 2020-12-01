// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

///! An example of using a custom storage adapter (in this case, using sled).
use iota_wallet::{
    account::AccountIdentifier, account_manager::AccountManager, client::ClientOptionsBuilder,
    storage::StorageAdapter,
};
use sled::Db;
use std::path::Path;

struct MyStorage {
    db: Db,
}

impl MyStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let instance = Self {
            db: sled::open(path)?,
        };
        Ok(instance)
    }
}

fn account_id_value(account_id: AccountIdentifier) -> anyhow::Result<String> {
    match account_id {
        AccountIdentifier::Id(val) => Ok(val),
        _ => Err(anyhow::anyhow!("Unexpected AccountIdentifier type")),
    }
}

impl StorageAdapter for MyStorage {
    fn get(&self, account_id: AccountIdentifier) -> iota_wallet::Result<String> {
        match self.db.get(account_id_value(account_id)?) {
            Ok(Some(value)) => Ok(String::from_utf8(value.to_vec()).unwrap()),
            Ok(None) => Err(anyhow::anyhow!("Value not found").into()),
            Err(e) => Err(anyhow::anyhow!("operational problem encountered: {}", e).into()),
        }
    }

    fn get_all(&self) -> iota_wallet::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        for tuple in self.db.iter() {
            let (_, value) = tuple.unwrap();
            accounts.push(String::from_utf8(value.to_vec()).unwrap());
        }
        Ok(accounts)
    }

    fn set(&self, account_id: AccountIdentifier, account: String) -> iota_wallet::Result<()> {
        self.db
            .insert(account_id_value(account_id)?, account.as_bytes())
            .map_err(|e| iota_wallet::WalletError::UnknownError(e.to_string()))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> iota_wallet::Result<()> {
        self.db
            .remove(account_id_value(account_id)?)
            .map_err(|e| iota_wallet::WalletError::UnknownError(e.to_string()))?;
        Ok(())
    }
}

fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::with_storage_adapter(
        "./example-database/sled",
        MyStorage::new("./example-database/sled")?,
    )
    .unwrap();
    manager.set_stronghold_password("password").unwrap();

    // first we'll create an example account
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    manager
        .create_account(client_options)
        .alias("alias")
        .initialise()?;

    Ok(())
}

///! An example of using a custom storage adapter (in this case, using rocksdb).
use iota_wallet::{
    account::AccountIdentifier, account_manager::AccountManager, client::ClientOptionsBuilder,
    storage::StorageAdapter,
};
use rocksdb::{IteratorMode, DB};
use std::path::Path;

struct MyStorage {
    db: DB,
}

impl MyStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let instance = Self {
            db: DB::open_default(path)?,
        };
        Ok(instance)
    }
}

fn account_id_value(account_id: AccountIdentifier) -> anyhow::Result<String> {
    match account_id {
        AccountIdentifier::Id(val) => Ok(String::from_utf8_lossy(&val).to_string()),
        _ => Err(anyhow::anyhow!("Unexpected AccountIdentifier type")),
    }
}

impl StorageAdapter for MyStorage {
    fn get(&self, account_id: AccountIdentifier) -> iota_wallet::Result<String> {
        match self.db.get(account_id_value(account_id)?) {
            Ok(Some(value)) => Ok(String::from_utf8(value).unwrap()),
            Ok(None) => Err(anyhow::anyhow!("Value not found").into()),
            Err(e) => Err(anyhow::anyhow!("operational problem encountered: {}", e).into()),
        }
    }

    fn get_all(&self) -> iota_wallet::Result<std::vec::Vec<String>> {
        let mut accounts = vec![];
        let iter = self.db.iterator(IteratorMode::Start);
        for (_, value) in iter {
            accounts.push(String::from_utf8(value.to_vec()).unwrap());
        }
        Ok(accounts)
    }

    fn set(&self, account_id: AccountIdentifier, account: String) -> iota_wallet::Result<()> {
        self.db
            .put(account_id_value(account_id)?, account)
            .map_err(|e| iota_wallet::WalletError::UnknownError(e.to_string()))?;
        Ok(())
    }

    fn remove(&self, account_id: AccountIdentifier) -> iota_wallet::Result<()> {
        self.db
            .delete(account_id_value(account_id)?)
            .map_err(|e| iota_wallet::WalletError::UnknownError(e.to_string()))?;
        Ok(())
    }
}

fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::with_storage_adapter(
        "./example-database/rocksdb",
        MyStorage::new("./example-database/rocksdb")?,
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

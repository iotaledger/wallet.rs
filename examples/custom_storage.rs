///! An example of using a custom storage adapter (in this case, using rocksdb).
use iota_wallet::{
  account::AccountIdentifier,
  account_manager::AccountManager,
  client::ClientOptionsBuilder,
  storage::{set_adapter, StorageAdapter},
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
    AccountIdentifier::Id(val) => Ok(val),
    _ => Err(anyhow::anyhow!("Unexpected AccountIdentifier type")),
  }
}

impl StorageAdapter for MyStorage {
  fn get(&self, account_id: AccountIdentifier) -> anyhow::Result<String> {
    match self.db.get(account_id_value(account_id)?) {
      Ok(Some(value)) => Ok(String::from_utf8(value).unwrap()),
      Ok(None) => Err(anyhow::anyhow!("Value not found")),
      Err(e) => Err(anyhow::anyhow!("operational problem encountered: {}", e)),
    }
  }

  fn get_all(&self) -> anyhow::Result<std::vec::Vec<String>> {
    let mut accounts = vec![];
    let iter = self.db.iterator(IteratorMode::Start);
    for (_, value) in iter {
      accounts.push(String::from_utf8(value.to_vec()).unwrap());
    }
    Ok(accounts)
  }

  fn set(&self, account_id: AccountIdentifier, account: String) -> anyhow::Result<()> {
    self.db.put(account_id_value(account_id)?, account)?;
    Ok(())
  }

  fn remove(&self, account_id: AccountIdentifier) -> anyhow::Result<()> {
    self.db.delete(account_id_value(account_id)?)?;
    Ok(())
  }
}

fn main() -> iota_wallet::Result<()> {
  // set the custom adapter
  set_adapter(MyStorage::new("./example-database/rocksdb")?)?;
  let manager = AccountManager::new();

  // first we'll create an example account
  let id = "test";
  let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
  manager
    .create_account(client_options)
    .alias(id)
    .id(id)
    .mnemonic(id)
    .initialise()?;

  Ok(())
}

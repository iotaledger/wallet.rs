## Database migration

Since there are breaking changes we need to migrate the database from wallet.rs.

Source for the following text: https://hackmd.io/Q3yQuUtvQLuis8putA9-aw?view

I have an idea in mind for database migrations when we need to (we might not need them for simple additions that has a [default value](https://serde.rs/attr-default.html)). The idea is simple: store a DB version record on RocksDB (like Bee does), and check that version on initialization. If we see an older version, we must apply the migrations manually, reading the record with the old schema and converting it to the new one. Here's a PoC code:

- Cargo.toml
```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2018"
[dependencies]
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.5", features = ["full"] }
rocksdb = { git="https://github.com/iotaledger/rust-rocksdb", rev = "70f2a53529ecc1853a2c025cec7f9d00bd50352c", default-features = false, features = ["lz4"] }
```
- main.rs
```rust
use rocksdb::{DBCompressionType, Options, WriteBatch, DB};
use tokio::sync::Mutex;
use std::{collections::HashMap, path::Path, sync::Arc};
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("recor not found")]
    RecordNotFound,
    #[error("{0}")]
    Storage(String),
}
pub type Result<T> = std::result::Result<T, Error>;
pub struct RocksdbStorageAdapter {
    db: Arc<Mutex<DB>>,
}
fn storage_err<E: ToString>(error: E) -> Error {
    Error::Storage(error.to_string())
}
impl RocksdbStorageAdapter {
    /// Initialises the storage adapter.
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut opts = Options::default();
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open(&opts, path).map_err(storage_err)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
    async fn get(&self, key: &str) -> Result<String> {
        match self.db.lock().await.get(key.as_bytes()) {
            Ok(Some(r)) => Ok(String::from_utf8_lossy(&r).to_string()),
            Ok(None) => Err(Error::RecordNotFound),
            Err(e) => Err(storage_err(e)),
        }
    }
    async fn set(&mut self, key: &str, record: String) -> Result<()> {
        self.db
            .lock()
            .await
            .put(key.as_bytes(), record.as_bytes())
            .map_err(storage_err)?;
        Ok(())
    }
    async fn batch_set(&mut self, records: HashMap<String, String>) -> Result<()> {
        let mut batch = WriteBatch::default();
        for (key, value) in records {
            batch.put(key.as_bytes(), value.as_bytes());
        }
        self.db.lock().await.write(batch).map_err(storage_err)?;
        Ok(())
    }
}
/// This is the public interface. It must match the latest version AccountDto.
#[derive(Debug, serde::Deserialize)]
struct Account {
    pub another: u64,
    pub value: u64,
}
mod v1 {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct AccountDto {
        pub something: u64,
    }
}
mod v2 {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct AccountDto {
        pub another: u64,
        pub value: u64,
    }
    impl AccountDto {
        // converts the v1 account schema to the v2 schema
        // here you can do anything, pull info from the node, etc
        pub async fn from_v1(account: super::v1::AccountDto) -> super::Result<Self> {
            Ok(Self {
                another: account.something,
                value: 5,
            })
        }
    }
}
const LATEST_DB_VERSION: usize = 2;
#[tokio::main]
async fn main() -> Result<()> {
    let mut storage = RocksdbStorageAdapter::new("./storage")?;
    // for this PoC we always have an account in storage so we can test the migration
    match storage.get("account").await {
        Ok(_) => {
            println!("Account record found");
        }
        Err(Error::RecordNotFound) => {
            println!("Account record not found. Creating account v1 for testing purposes");
            let v1_account = v1::AccountDto { something: 1 };
            storage
                .set("account", serde_json::to_string(&v1_account).unwrap())
                .await?;
        }
        Err(e) => return Err(e),
    }
    let version = match storage.get("version").await {
        Ok(v) => serde_json::from_str(&v).unwrap(),
        Err(Error::RecordNotFound) => {
            println!("version not found; assuming `1`");
            storage.set("version", "1".to_string()).await?;
            1usize
        }
        Err(e) => return Err(e),
    };
    println!("DB version: {}", version);
    if version == LATEST_DB_VERSION {
        println!("Already on latest DB version");
    } else {
        for v in version..LATEST_DB_VERSION {
            match v {
                1 => {
                    println!("migrating from v1 to v2");
                    migrate_to_v2(&mut storage).await?;
                }
                _ => panic!("unexpected version {}", v),
            }
        }
        // we've migrated to latest; update record
        storage
            .set("version", LATEST_DB_VERSION.to_string())
            .await?;
    }
    // get the migrated account
    let account: Account = serde_json::from_str(&storage.get("account").await?).unwrap();
    println!("Account: {:?}", account);
    Ok(())
}
async fn migrate_to_v2(storage: &mut RocksdbStorageAdapter) -> Result<()> {
    let mut data_to_save = HashMap::new();
    // get the v1 account
    let account = storage.get("account").await?;
    // convert to v2
    let accountv2 = v2::AccountDto::from_v1(serde_json::from_str(&account).unwrap()).await?;
    // save
    data_to_save.insert(
        "account".to_string(),
        serde_json::to_string(&accountv2).unwrap(),
    );
    // use batch_set so it's atomic
    storage.batch_set(data_to_save).await?;
    Ok(())
}
```

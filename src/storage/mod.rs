// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "sqlite-storage")]
/// Sqlite storage.
pub mod sqlite;

#[cfg(any(feature = "stronghold-storage", feature = "stronghold"))]
/// Stronghold storage.
pub mod stronghold;

use crate::account::{Account, AccountIdentifier};
use crypto::ciphers::chacha::xchacha20poly1305;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex as AsyncMutex;

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

type Storage = Arc<AsyncMutex<Box<dyn StorageAdapter + Sync + Send>>>;
type Storages = Arc<RwLock<HashMap<PathBuf, Storage>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub fn set_adapter<P: AsRef<Path>, S: StorageAdapter + Sync + Send + 'static>(storage_path: P, storage: S) {
    let mut instances = INSTANCES.get_or_init(Default::default).write().unwrap();
    instances.insert(
        storage_path.as_ref().to_path_buf(),
        Arc::new(AsyncMutex::new(Box::new(storage))),
    );
}

/// gets the storage adapter
pub(crate) fn get(storage_path: &PathBuf) -> crate::Result<Storage> {
    let instances = INSTANCES.get_or_init(Default::default).read().unwrap();
    if let Some(instance) = instances.get(storage_path) {
        Ok(instance.clone())
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.clone()))
    }
}

/// The storage adapter.
#[async_trait::async_trait]
pub trait StorageAdapter {
    /// Gets the storage identifier (used internally on the default storage adapters)
    fn id(&self) -> &'static str {
        "custom-adapter"
    }
    /// Gets the account with the given id/alias from the storage.
    async fn get(&self, account_id: &AccountIdentifier) -> crate::Result<String>;
    /// Gets all the accounts from the storage.
    async fn get_all(&self) -> crate::Result<Vec<String>>;
    /// Saves or updates an account on the storage.
    async fn set(&self, account_id: &AccountIdentifier, account: String) -> crate::Result<()>;
    /// Removes an account from the storage.
    async fn remove(&self, account_id: &AccountIdentifier) -> crate::Result<()>;
}

fn encrypt_account_json<O: Write>(account: &[u8], key: &[u8; 32], output: &mut O) -> crate::Result<()> {
    let mut nonce = [0; xchacha20poly1305::XCHACHA20POLY1305_NONCE_SIZE];
    crypto::rand::fill(&mut nonce)?;

    let mut tag = [0; xchacha20poly1305::XCHACHA20POLY1305_TAG_SIZE];
    let mut ct = vec![0; account.len()];
    xchacha20poly1305::encrypt(&mut ct, &mut tag, account, key, &nonce, &[])?;

    output.write_all(&nonce)?;
    output.write_all(&tag)?;
    output.write_all(&ct)?;

    Ok(())
}

pub(crate) fn decrypt_account_json(account: &str, key: &[u8; 32]) -> crate::Result<String> {
    let account: Vec<u8> = serde_json::from_str(&account)?;
    let mut account: &[u8] = &account;

    let mut nonce = [0; xchacha20poly1305::XCHACHA20POLY1305_NONCE_SIZE];
    account.read_exact(&mut nonce)?;

    let mut tag = [0; xchacha20poly1305::XCHACHA20POLY1305_TAG_SIZE];
    account.read_exact(&mut tag)?;

    let mut ct = Vec::new();
    account.read_to_end(&mut ct)?;

    let mut pt = vec![0; ct.len()];
    xchacha20poly1305::decrypt(&mut pt, &ct, key, &tag, &nonce, &[])?;

    Ok(String::from_utf8_lossy(&pt).to_string())
}

pub(crate) enum ParsedAccount {
    Account(Account),
    EncryptedAccount(String),
}

pub(crate) fn get_account_string_to_save(
    account: &Account,
    encryption_key: &Option<[u8; 32]>,
) -> crate::Result<String> {
    let json = serde_json::to_string(&account)?;
    let data = if let Some(key) = encryption_key {
        let mut output = Vec::new();
        encrypt_account_json(json.as_bytes(), key, &mut output)?;
        serde_json::to_string(&output)?
    } else {
        json
    };
    Ok(data)
}

pub(crate) async fn save_account(
    storage_path: &PathBuf,
    account: &Account,
    encryption_key: &Option<[u8; 32]>,
) -> crate::Result<()> {
    crate::storage::get(&storage_path)?
        .lock()
        .await
        .set(account.id(), get_account_string_to_save(account, encryption_key)?)
        .await?;
    Ok(())
}

pub(crate) fn parse_accounts(
    storage_path: &PathBuf,
    accounts: &[String],
    encryption_key: &Option<[u8; 32]>,
) -> crate::Result<Vec<ParsedAccount>> {
    let mut err = None;
    let accounts: Vec<Option<ParsedAccount>> = accounts
        .iter()
        .map(|account| {
            let account_json = if account.starts_with('{') {
                Some(account.to_string())
            } else if let Some(key) = encryption_key {
                if let Ok(json) = decrypt_account_json(account, key) {
                    Some(json)
                } else {
                    err = Some(crate::Error::AccountDecrypt);
                    return None;
                }
            } else {
                None
            };
            if let Some(json) = account_json {
                match serde_json::from_str::<Account>(&json) {
                    Ok(mut acc) => {
                        acc.set_storage_path(storage_path.clone());
                        Some(ParsedAccount::Account(acc))
                    }
                    Err(e) => {
                        err = Some(e.into());
                        None
                    }
                }
            } else {
                Some(ParsedAccount::EncryptedAccount(account.to_string()))
            }
        })
        .collect();

    if let Some(err) = err {
        Err(err)
    } else {
        let accounts = accounts.into_iter().map(|account| account.unwrap()).collect();
        Ok(accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::StorageAdapter;
    use crate::account::AccountIdentifier;
    use std::path::PathBuf;

    #[tokio::test]
    // asserts that the adapter defined by `set_adapter` is globally available with `get_adapter`
    async fn set_adapter() {
        struct MyAdapter;
        #[async_trait::async_trait]
        impl StorageAdapter for MyAdapter {
            async fn get(&self, _key: &AccountIdentifier) -> crate::Result<String> {
                Ok("MY_ADAPTER_GET_RESPONSE".to_string())
            }
            async fn get_all(&self) -> crate::Result<Vec<String>> {
                Ok(vec![])
            }
            async fn set(&self, _key: &AccountIdentifier, _account: String) -> crate::Result<()> {
                Ok(())
            }
            async fn remove(&self, _key: &AccountIdentifier) -> crate::Result<()> {
                Ok(())
            }
        }

        let path = "./the-storage-path";
        super::set_adapter(path, MyAdapter {});
        let adapter = super::get(&std::path::PathBuf::from(path)).unwrap();
        let adapter = adapter.lock().await;
        assert_eq!(
            adapter.get(&"".to_string().into()).await.unwrap(),
            "MY_ADAPTER_GET_RESPONSE".to_string()
        );
    }

    #[test]
    fn parse_accounts_invalid() {
        let response = super::parse_accounts(&PathBuf::new(), &["{}".to_string()], &None);
        assert!(response.is_err());
    }

    async fn _create_account() -> (std::path::PathBuf, crate::account::AccountHandle) {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = crate::client::ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .unwrap()
            .build();
        let account_handle = manager
            .create_account(client_options)
            .alias("alias")
            .initialise()
            .await
            .unwrap();
        (manager.storage_path().clone(), account_handle)
    }

    #[tokio::test]
    async fn parse_accounts_valid() {
        let (storage_path, account_handle) = _create_account().await;
        let response = super::parse_accounts(
            &storage_path,
            &[serde_json::to_string(&*account_handle.read().await).unwrap()],
            &None,
        );
        assert!(response.is_ok());
        let parsed_accounts = response.unwrap();
        let parsed_account = parsed_accounts.first().unwrap();
        match parsed_account {
            super::ParsedAccount::Account(parsed) => assert_eq!(parsed, &*account_handle.read().await),
            _ => panic!("invalid parsed account format"),
        }
    }
}

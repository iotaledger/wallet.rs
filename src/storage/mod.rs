// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "sqlite-storage")]
/// Sqlite storage.
pub mod sqlite;

#[cfg(any(feature = "stronghold-storage", feature = "stronghold"))]
/// Stronghold storage.
pub mod stronghold;

use crate::account::Account;
use crypto::ciphers::chacha::xchacha20poly1305;
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex, RwLock};

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

pub(crate) struct Storage {
    storage_path: PathBuf,
    inner: Box<dyn StorageAdapter + Sync + Send>,
    pub(crate) encryption_key: Option<[u8; 32]>,
}

impl Storage {
    pub fn id(&self) -> &'static str {
        self.inner.id()
    }

    #[allow(dead_code)]
    pub async fn get(&mut self, account_id: &str) -> crate::Result<String> {
        self.inner.get(account_id).await.and_then(|account| {
            if let Some(key) = &self.encryption_key {
                decrypt_account_json(&account, key)
            } else {
                Ok(account)
            }
        })
    }

    pub async fn get_all(&mut self) -> crate::Result<Vec<ParsedAccount>> {
        parse_accounts(&self.storage_path, &self.inner.get_all().await?, &self.encryption_key)
    }

    pub async fn set(&mut self, account_id: &str, account: String) -> crate::Result<()> {
        self.inner
            .set(
                account_id,
                if let Some(key) = &self.encryption_key {
                    let mut output = Vec::new();
                    encrypt_account_json(account.as_bytes(), key, &mut output)?;
                    serde_json::to_string(&output)?
                } else {
                    account
                },
            )
            .await
    }

    pub async fn remove(&mut self, account_id: &str) -> crate::Result<()> {
        self.inner.remove(account_id).await
    }
}

type StorageHandle = Arc<Mutex<Storage>>;
type Storages = Arc<RwLock<HashMap<PathBuf, StorageHandle>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub async fn set<P: AsRef<Path>>(
    storage_path: P,
    encryption_key: Option<[u8; 32]>,
    storage: Box<dyn StorageAdapter + Send + Sync + 'static>,
) {
    let mut instances = INSTANCES.get_or_init(Default::default).write().await;
    instances.insert(
        storage_path.as_ref().to_path_buf(),
        Arc::new(Mutex::new(Storage {
            storage_path: storage_path.as_ref().to_path_buf(),
            inner: storage,
            encryption_key,
        })),
    );
}

pub(crate) async fn set_encryption_key(storage_path: &PathBuf, encryption_key: [u8; 32]) -> crate::Result<()> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
    if let Some(instance) = instances.get(storage_path) {
        let mut storage = instance.lock().await;
        storage.encryption_key = Some(encryption_key);
        Ok(())
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.clone()))
    }
}

/// gets the storage adapter
pub(crate) async fn get(storage_path: &PathBuf) -> crate::Result<StorageHandle> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
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
    async fn get(&mut self, account_id: &str) -> crate::Result<String>;
    /// Gets all the accounts from the storage.
    async fn get_all(&mut self) -> crate::Result<Vec<String>>;
    /// Saves or updates an account on the storage.
    async fn set(&mut self, account_id: &str, account: String) -> crate::Result<()>;
    /// Removes an account from the storage.
    async fn remove(&mut self, account_id: &str) -> crate::Result<()>;
}

fn encrypt_account_json<O: Write>(account: &[u8], key: &[u8; 32], output: &mut O) -> crate::Result<()> {
    let mut nonce = [0; xchacha20poly1305::XCHACHA20POLY1305_NONCE_SIZE];
    crypto::rand::fill(&mut nonce).map_err(|e| crate::Error::AccountEncrypt(format!("{:?}", e)))?;

    let mut tag = [0; xchacha20poly1305::XCHACHA20POLY1305_TAG_SIZE];
    let mut ct = vec![0; account.len()];
    xchacha20poly1305::encrypt(&mut ct, &mut tag, account, key, &nonce, &[])
        .map_err(|e| crate::Error::AccountEncrypt(format!("{:?}", e)))?;

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
    xchacha20poly1305::decrypt(&mut pt, &ct, key, &tag, &nonce, &[])
        .map_err(|e| crate::Error::AccountDecrypt(format!("{:?}", e)))?;

    Ok(String::from_utf8_lossy(&pt).to_string())
}

pub(crate) enum ParsedAccount {
    Account(Account),
    EncryptedAccount(String),
}

fn parse_accounts(
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
                match decrypt_account_json(account, key) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        err = Some(e);
                        None
                    }
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

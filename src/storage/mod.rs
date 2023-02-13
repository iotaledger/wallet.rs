// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Storage adapter.
pub mod adapter;
/// Storage constants.
pub mod constants;
/// Storage manager.
pub mod manager;
/// Storage functions related to participation.
#[cfg(feature = "participation")]
mod participation;

use std::collections::HashMap;

use crypto::ciphers::chacha;
use serde::Serialize;

use self::adapter::StorageAdapter;

#[derive(Debug)]
pub(crate) struct Storage {
    inner: Box<dyn StorageAdapter + Sync + Send>,
    encryption_key: Option<[u8; 32]>,
}

impl Storage {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    async fn get(&self, key: &str) -> crate::Result<Option<String>> {
        let record = self.inner.get(key).await?;

        match record {
            Some(record) => {
                if let Some(key) = &self.encryption_key {
                    if serde_json::from_str::<Vec<u8>>(&record).is_ok() {
                        Ok(Some(
                            String::from_utf8_lossy(&chacha::aead_decrypt(key, record.as_bytes())?).into_owned(),
                        ))
                    } else {
                        Ok(Some(record))
                    }
                } else {
                    Ok(Some(record))
                }
            }
            None => return Ok(None),
        }
    }

    async fn set<T: Serialize + Send>(&mut self, key: &str, record: T) -> crate::Result<()> {
        let record = serde_json::to_string(&record)?;
        self.inner
            .set(
                key,
                if let Some(key) = &self.encryption_key {
                    let output = chacha::aead_encrypt(key, record.as_bytes())?;
                    serde_json::to_string(&output)?
                } else {
                    record
                },
            )
            .await
    }

    #[allow(dead_code)]
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        self.inner
            .batch_set(if let Some(key) = &self.encryption_key {
                let mut encrypted_records = HashMap::new();
                for (id, record) in records {
                    let output = chacha::aead_encrypt(key, record.as_bytes())?;
                    encrypted_records.insert(id, serde_json::to_string(&output)?);
                }
                encrypted_records
            } else {
                records
            })
            .await
    }

    async fn remove(&mut self, key: &str) -> crate::Result<()> {
        self.inner.remove(key).await
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        log::debug!("drop Storage");
    }
}

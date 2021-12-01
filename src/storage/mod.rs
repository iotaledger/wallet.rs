// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Storage adapter.
pub mod adapter;
/// Storage constants.
pub mod constants;
/// Storage encryption.
pub mod encryption;
/// Storage manager.
pub mod manager;

use adapter::StorageAdapter;
use encryption::{decrypt_record, encrypt_record};

use serde::Serialize;

use std::{collections::HashMap, path::PathBuf};

struct Storage {
    storage_path: PathBuf,
    inner: Box<dyn StorageAdapter + Sync + Send>,
    encryption_key: Option<[u8; 32]>,
}

impl Storage {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    async fn get(&self, key: &str) -> crate::Result<String> {
        self.inner.get(key).await.and_then(|record| {
            if let Some(key) = &self.encryption_key {
                if serde_json::from_str::<Vec<u8>>(&record).is_ok() {
                    decrypt_record(&record, key)
                } else {
                    Ok(record)
                }
            } else {
                Ok(record)
            }
        })
    }

    async fn set<T: Serialize>(&mut self, key: &str, record: T) -> crate::Result<()> {
        let record = serde_json::to_string(&record)?;
        self.inner
            .set(
                key,
                if let Some(key) = &self.encryption_key {
                    let mut output = Vec::new();
                    encrypt_record(record.as_bytes(), key, &mut output)?;
                    serde_json::to_string(&output)?
                } else {
                    record
                },
            )
            .await
    }

    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
        self.inner
            .batch_set(if let Some(key) = &self.encryption_key {
                let mut encrypted_records = HashMap::new();
                for (id, record) in records {
                    let mut output = Vec::new();
                    encrypt_record(record.as_bytes(), key, &mut output)?;
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

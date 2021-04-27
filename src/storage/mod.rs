// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// RocksDB storage.
pub mod rocksdb;

/// Stronghold storage.
pub mod stronghold;

use crate::{
    account::Account,
    event::{BalanceEvent, TransactionConfirmationChangeEvent, TransactionEvent, TransactionReattachmentEvent},
    message::{Message, MessageId, MessagePayload, MessageType, TransactionEssence},
};

use chrono::Utc;
use crypto::ciphers::{chacha::XChaCha20Poly1305, traits::Aead};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use std::{
    collections::HashMap,
    convert::TryInto,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

const ACCOUNT_INDEXATION_KEY: &str = "iota-wallet-account-indexation";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AccountIndexation {
    key: String,
}

pub(crate) type Timestamp = i64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EventIndexation {
    key: String,
    timestamp: Timestamp,
}

/// The indexation for account messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageIndexation {
    /// The message id.
    pub key: MessageId,
    /// The payload hash.
    pub payload_hash: Option<u64>,
    pub internal: Option<bool>,
    /// Whether the message is an incoming or an outgoing transaction.
    pub incoming: Option<bool>,
    /// Whether the message was broadcasted or not.
    pub broadcasted: bool,
    /// Whether the message was confirmed or not.
    /// None means that it is still pending.
    pub confirmed: Option<bool>,
    /// Message value.
    pub value: u64,
    /// Id of the message that reattached this message.
    pub reattachment_message_id: Option<MessageId>,
}

#[derive(Default)]
pub struct MessageQueryFilter<'a> {
    message_type: Option<MessageType>,
    ignore_ids: Option<&'a HashMap<MessageId, Message>>,
}

impl<'a> MessageQueryFilter<'a> {
    pub fn message_type(message_type: Option<MessageType>) -> Self {
        Self {
            message_type,
            ignore_ids: Default::default(),
        }
    }

    pub fn with_ignore_ids(mut self, ignore_ids: &'a HashMap<MessageId, Message>) -> Self {
        self.ignore_ids.replace(ignore_ids);
        self
    }
}

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

pub(crate) struct StorageManager {
    storage: Storage,
    account_indexation: Vec<AccountIndexation>,
    message_indexation: HashMap<String, Vec<MessageIndexation>>,
    balance_change_indexation: Option<Vec<EventIndexation>>,
    transaction_confirmation_indexation: Option<Vec<EventIndexation>>,
    new_transaction_indexation: Option<Vec<EventIndexation>>,
    reattachment_indexation: Option<Vec<EventIndexation>>,
    broadcast_indexation: Option<Vec<EventIndexation>>,
}

macro_rules! load_account_dependency_index {
    ($self: ident, $account_id: expr, $key: expr, $indexation: ident) => {
        match $self.storage.get($key).await {
            Ok(record) => {
                $self.$indexation.insert($account_id, serde_json::from_str(&record)?);
            }
            Err(crate::Error::RecordNotFound) => {
                $self.$indexation.insert($account_id, Default::default());
            }
            Err(e) => return Err(e),
        }
    };
}

macro_rules! init_account_dependency_index {
    ($self: ident, $account_id: expr, $indexation: ident) => {
        if !$self.$indexation.contains_key($account_id) {
            $self.$indexation.insert($account_id.to_string(), Default::default());
        }
    };
}

fn account_message_index_key(account_id: &str) -> String {
    format!("iota-wallet-{}-messages", account_id)
}

impl StorageManager {
    pub fn id(&self) -> &'static str {
        self.storage.id()
    }

    #[cfg(test)]
    pub fn is_encrypted(&self) -> bool {
        self.storage.encryption_key.is_some()
    }

    pub async fn get(&self, key: &str) -> crate::Result<String> {
        self.storage.get(key).await
    }

    pub async fn get_accounts(&mut self) -> crate::Result<Vec<Account>> {
        if self.account_indexation.is_empty() {
            if let Ok(record) = self.storage.get(ACCOUNT_INDEXATION_KEY).await {
                self.account_indexation = serde_json::from_str(&record)?;
            }
        }

        let mut accounts = Vec::new();
        for account_index in self.account_indexation.clone() {
            accounts.push(self.get(&account_index.key).await?);
            load_account_dependency_index!(
                self,
                account_index.key.clone(),
                &account_message_index_key(&account_index.key),
                message_indexation
            );
        }
        parse_accounts(&self.storage.storage_path, &accounts, &self.storage.encryption_key)
    }

    pub async fn save_account(&mut self, key: &str, account: &Account) -> crate::Result<()> {
        let index = AccountIndexation { key: key.to_string() };
        self.storage.set(key, account).await?;
        if !self.account_indexation.contains(&index) {
            init_account_dependency_index!(self, key, message_indexation);
            self.account_indexation.push(index);
            self.storage
                .set(ACCOUNT_INDEXATION_KEY, &self.account_indexation)
                .await?;
        }
        Ok(())
    }

    pub async fn remove_account(&mut self, key: &str) -> crate::Result<()> {
        let index = AccountIndexation { key: key.to_string() };
        if let Some(index) = self.account_indexation.iter().position(|i| i == &index) {
            self.account_indexation.remove(index);
            self.storage
                .set(ACCOUNT_INDEXATION_KEY, &self.account_indexation)
                .await?;
            self.storage.remove(key).await?;
            Ok(())
        } else {
            Err(crate::Error::RecordNotFound)
        }
    }

    pub fn message_indexation(&self, account: &Account) -> crate::Result<&Vec<MessageIndexation>> {
        self.message_indexation
            .get(account.id())
            .ok_or(crate::Error::RecordNotFound)
    }

    pub fn query_message_indexation(
        &self,
        account: &Account,
        filter: &MessageQueryFilter<'_>,
    ) -> crate::Result<Vec<&MessageIndexation>> {
        let message_indexation = self.message_indexation(account)?;

        let mut filtered_message_indexation = Vec::new();
        for message in message_indexation {
            if message.reattachment_message_id.is_some() {
                continue;
            }
            let message_type_matches = if let Some(message_type) = filter.message_type.clone() {
                match message_type {
                    MessageType::Received => message.incoming == Some(true),
                    MessageType::Sent => message.incoming == Some(false),
                    MessageType::Failed => !message.broadcasted,
                    MessageType::Unconfirmed => message.confirmed.is_none(),
                    MessageType::Value => message.value > 0,
                    MessageType::Confirmed => message.confirmed.is_some(),
                }
            } else {
                true
            };
            if message_type_matches {
                filtered_message_indexation.push(message);
            }
        }

        Ok(filtered_message_indexation)
    }

    pub async fn save_messages(&mut self, account: &Account, messages: &[Message]) -> crate::Result<()> {
        let message_indexation = self
            .message_indexation
            .entry(account.id().clone())
            .or_insert_with(Default::default);
        let mut messages_map = HashMap::new();
        for message in messages.iter() {
            messages_map.insert(message.id().to_string(), serde_json::to_string(&message)?);
            let (value, internal, incoming) = match message.payload() {
                Some(MessagePayload::Transaction(tx)) => {
                    let TransactionEssence::Regular(essence) = tx.essence();
                    (
                        essence.value(),
                        Some(essence.internal()),
                        // recompute the `incoming` flag on the indexation
                        Some(essence.is_incoming(account.addresses())),
                    )
                }
                _ => (0, None, None),
            };
            let index = MessageIndexation {
                key: *message.id(),
                payload_hash: message.payload().as_ref().map(|p| p.storage_hash()),
                internal,
                incoming,
                broadcasted: message.broadcasted,
                confirmed: message.confirmed,
                value,
                reattachment_message_id: None,
            };
            if let Some(position) = message_indexation.iter().position(|i| i.key == index.key) {
                message_indexation[position] = index.clone();
            } else {
                message_indexation.push(index.clone());
            }
        }
        self.storage
            .set(&account_message_index_key(account.id()), &message_indexation)
            .await?;
        self.storage.batch_set(messages_map).await?;
        Ok(())
    }

    pub async fn get_message(&self, account: &Account, message_id: &MessageId) -> crate::Result<Message> {
        let message_indexation = self.message_indexation(account)?;
        let index = message_indexation
            .iter()
            .find(|i| &i.key == message_id)
            .ok_or(crate::Error::RecordNotFound)?;
        let message = self.get(&index.key.to_string()).await?;
        serde_json::from_str(&message).map_err(Into::into)
    }

    pub async fn get_messages(
        &self,
        account: &Account,
        count: usize,
        skip: usize,
        filter: MessageQueryFilter<'_>,
    ) -> crate::Result<Vec<Message>> {
        let filtered_message_indexation = self.query_message_indexation(account, &filter)?;

        let mut messages = Vec::new();
        let iter = filtered_message_indexation.into_iter().skip(skip);
        for index in if count == 0 {
            iter.collect::<Vec<&MessageIndexation>>()
        } else {
            iter.take(count).collect::<Vec<&MessageIndexation>>()
        } {
            let message: Option<Message> = match &filter.ignore_ids {
                Some(ignore_ids) => {
                    if !ignore_ids.contains_key(&index.key) {
                        let message = self.get(&index.key.to_string()).await?;
                        Some(serde_json::from_str(&message)?)
                    } else {
                        None
                    }
                }
                None => {
                    let message = self.get(&index.key.to_string()).await?;
                    Some(serde_json::from_str(&message)?)
                }
            };
            if let Some(mut message) = message {
                // we update the `incoming` prop because we store only one copy of the message on the db
                // so on internal transactions the `incoming` prop is wrong without this
                if let Some(MessagePayload::Transaction(tx)) = message.payload.as_mut() {
                    let TransactionEssence::Regular(essence) = tx.essence_mut();
                    essence.incoming = index.incoming.unwrap_or_default();
                }
                messages.push(message);
            }
        }
        Ok(messages)
    }
}

async fn load_optional_data<T: DeserializeOwned + Default>(storage: &Storage, key: &str) -> crate::Result<T> {
    let record = match storage.get(key).await {
        Ok(record) => serde_json::from_str(&record)?,
        Err(crate::Error::RecordNotFound) => T::default(),
        Err(e) => return Err(e),
    };
    Ok(record)
}

macro_rules! event_manager_impl {
    ($event_ty:ty, $index_vec:ident, $index_key: expr, $save_fn_name: ident, $get_fn_name: ident, $get_count_fn_name: ident) => {
        impl StorageManager {
            pub async fn $save_fn_name(&mut self, event: &$event_ty) -> crate::Result<()> {
                let key = event.indexation_id.clone();
                self.storage.set(&key, event).await?;
                let index = EventIndexation {
                    key: key.to_string(),
                    timestamp: Utc::now().timestamp(),
                };
                match self.$index_vec {
                    Some(ref mut indexation) => indexation.push(index),
                    None => {
                        let mut indexation: Vec<EventIndexation> =
                            load_optional_data(&self.storage, $index_key).await?;
                        indexation.push(index);
                        self.$index_vec.replace(indexation);
                    }
                }
                self.storage.set($index_key, &self.$index_vec).await?;
                Ok(())
            }

            pub async fn $get_fn_name<T: Into<Option<Timestamp>>>(
                &mut self,
                count: usize,
                skip: usize,
                from_timestamp: T,
            ) -> crate::Result<Vec<$event_ty>> {
                let indexation = match &self.$index_vec {
                    Some(indexation) => indexation,
                    None => {
                        self.$index_vec
                            .replace(load_optional_data(&self.storage, $index_key).await?);
                        self.$index_vec.as_ref().unwrap()
                    }
                };
                let mut events = Vec::new();
                let from_timestamp = from_timestamp.into().unwrap_or(0);
                let iter = indexation
                    .iter()
                    .filter(|i| i.timestamp >= from_timestamp)
                    .skip(skip);
                for index in if count == 0 {
                    iter.collect::<Vec<&EventIndexation>>()
                } else {
                    iter.take(count).collect::<Vec<&EventIndexation>>()
                } {
                    let event_json = self.get(&index.key).await?;
                    events.push(serde_json::from_str(&event_json)?);
                }
                Ok(events)
            }

            pub async fn $get_count_fn_name<T: Into<Option<Timestamp>>>(
                &mut self,
                from_timestamp: T,
            ) -> crate::Result<usize> {
                let indexation = match &self.$index_vec {
                    Some(indexation) => indexation,
                    None => {
                        self.$index_vec
                            .replace(load_optional_data(&self.storage, $index_key).await?);
                        self.$index_vec.as_ref().unwrap()
                    }
                };
                let from_timestamp = from_timestamp.into().unwrap_or(0);
                let count = indexation
                    .iter()
                    // using folder since it's faster than .filter().count()
                    .fold(0, |count, index| {
                        if index.timestamp >= from_timestamp {
                            count + 1
                        } else {
                            count
                        }
                    });
                Ok(count)
            }
        }
    };
}

event_manager_impl!(
    BalanceEvent,
    balance_change_indexation,
    "iota-wallet-balance-change-events",
    save_balance_change_event,
    get_balance_change_events,
    get_balance_change_event_count
);
event_manager_impl!(
    TransactionConfirmationChangeEvent,
    transaction_confirmation_indexation,
    "iota-wallet-tx-confirmation-events",
    save_transaction_confirmation_event,
    get_transaction_confirmation_events,
    get_transaction_confirmation_event_count
);
event_manager_impl!(
    TransactionEvent,
    new_transaction_indexation,
    "iota-wallet-new-tx-events",
    save_new_transaction_event,
    get_new_transaction_events,
    get_new_transaction_event_count
);
event_manager_impl!(
    TransactionReattachmentEvent,
    reattachment_indexation,
    "iota-wallet-tx-reattachment-events",
    save_reattachment_event,
    get_reattachment_events,
    get_reattachment_event_count
);
event_manager_impl!(
    TransactionEvent,
    broadcast_indexation,
    "iota-wallet-tx-broadcast-events",
    save_broadcast_event,
    get_broadcast_events,
    get_broadcast_event_count
);

pub(crate) type StorageHandle = Arc<Mutex<StorageManager>>;
type Storages = Arc<RwLock<HashMap<PathBuf, StorageHandle>>>;
static INSTANCES: OnceCell<Storages> = OnceCell::new();

/// Sets the storage adapter.
pub(crate) async fn set<P: AsRef<Path>>(
    storage_path: P,
    encryption_key: Option<[u8; 32]>,
    storage: Box<dyn StorageAdapter + Send + Sync + 'static>,
) {
    let mut instances = INSTANCES.get_or_init(Default::default).write().await;
    #[allow(unused_variables)]
    let storage_id = storage.id();
    let storage = Storage {
        storage_path: storage_path.as_ref().to_path_buf(),
        inner: storage,
        encryption_key: if storage_id == stronghold::STORAGE_ID {
            None
        } else {
            encryption_key
        },
    };
    let storage_manager = StorageManager {
        storage,
        account_indexation: Default::default(),
        message_indexation: Default::default(),
        balance_change_indexation: Default::default(),
        transaction_confirmation_indexation: Default::default(),
        new_transaction_indexation: Default::default(),
        reattachment_indexation: Default::default(),
        broadcast_indexation: Default::default(),
    };
    instances.insert(
        storage_path.as_ref().to_path_buf(),
        Arc::new(Mutex::new(storage_manager)),
    );
}

pub(crate) async fn remove(storage_path: &Path) -> Option<String> {
    let mut instances = INSTANCES.get_or_init(Default::default).write().await;
    let storage = instances.remove(storage_path);
    if let Some(s) = storage {
        Some(s.lock().await.id().to_string())
    } else {
        None
    }
}

pub(crate) async fn set_encryption_key(storage_path: &Path, encryption_key: [u8; 32]) -> crate::Result<()> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
    if let Some(instance) = instances.get(storage_path) {
        let mut storage_manager = instance.lock().await;
        storage_manager.storage.encryption_key.replace(encryption_key);
        Ok(())
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.to_path_buf()))
    }
}

/// gets the storage adapter
pub(crate) async fn get(storage_path: &Path) -> crate::Result<StorageHandle> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
    if let Some(instance) = instances.get(storage_path) {
        Ok(instance.clone())
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.to_path_buf()))
    }
}

/// The storage adapter.
#[async_trait::async_trait]
pub trait StorageAdapter {
    /// Gets the storage identifier (used internally on the default storage adapters)
    fn id(&self) -> &'static str {
        "custom-adapter"
    }
    /// Gets the record associated with the given key from the storage.
    async fn get(&self, key: &str) -> crate::Result<String>;
    /// Saves or updates a record on the storage.
    async fn set(&mut self, key: &str, record: String) -> crate::Result<()>;
    /// Batch write.
    async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()>;
    /// Removes a record from the storage.
    async fn remove(&mut self, key: &str) -> crate::Result<()>;
}

fn encrypt_record<O: Write>(record: &[u8], encryption_key: &[u8; 32], output: &mut O) -> crate::Result<()> {
    let mut nonce = [0; XChaCha20Poly1305::NONCE_LENGTH];
    crypto::utils::rand::fill(&mut nonce).map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;

    let mut tag = vec![0; XChaCha20Poly1305::TAG_LENGTH];
    let mut ciphertext = vec![0; record.len()];
    // we can unwrap here since we know the lengths are valid
    XChaCha20Poly1305::encrypt(
        encryption_key.try_into().unwrap(),
        &nonce.try_into().unwrap(),
        &[],
        record,
        &mut ciphertext,
        tag.as_mut_slice().try_into().unwrap(),
    )
    .map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;

    output.write_all(&nonce)?;
    output.write_all(&tag)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub(crate) fn decrypt_record(record: &str, encryption_key: &[u8; 32]) -> crate::Result<String> {
    let record: Vec<u8> = serde_json::from_str(&record)?;
    let mut record: &[u8] = &record;

    let mut nonce = [0; XChaCha20Poly1305::NONCE_LENGTH];
    record.read_exact(&mut nonce)?;

    let mut tag = vec![0; XChaCha20Poly1305::TAG_LENGTH];
    record.read_exact(&mut tag)?;

    let mut ct = Vec::new();
    record.read_to_end(&mut ct)?;

    let mut pt = vec![0; ct.len()];
    // we can unwrap here since we know the lengths are valid
    XChaCha20Poly1305::decrypt(
        encryption_key.try_into().unwrap(),
        &nonce.try_into().unwrap(),
        &[],
        &mut pt,
        &ct,
        tag.as_slice().try_into().unwrap(),
    )
    .map_err(|e| crate::Error::RecordDecrypt(format!("{:?}", e)))?;

    Ok(String::from_utf8_lossy(&pt).to_string())
}

fn parse_accounts(
    storage_path: &Path,
    accounts: &[String],
    encryption_key: &Option<[u8; 32]>,
) -> crate::Result<Vec<Account>> {
    let mut err = None;
    let accounts: Vec<Option<Account>> = accounts
        .iter()
        .map(|account| {
            let account_json = if account.starts_with('{') {
                Some(account.to_string())
            } else if let Some(key) = encryption_key {
                match decrypt_record(account, key) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        err.replace(e);
                        None
                    }
                }
            } else {
                None
            };
            if let Some(json) = account_json {
                match serde_json::from_str::<Account>(&json) {
                    Ok(mut acc) => {
                        acc.set_storage_path(storage_path.to_path_buf());
                        Some(acc)
                    }
                    Err(e) => {
                        err.replace(e.into());
                        None
                    }
                }
            } else {
                err.replace(crate::Error::StorageIsEncrypted);
                None
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
    use std::{collections::HashMap, path::PathBuf};

    #[tokio::test]
    // asserts that the adapter defined by `set` is globally available with `get`
    async fn set_adapter() {
        struct MyAdapter;
        #[async_trait::async_trait]
        impl StorageAdapter for MyAdapter {
            async fn get(&self, _key: &str) -> crate::Result<String> {
                Ok("MY_ADAPTER_GET_RESPONSE".to_string())
            }
            async fn set(&mut self, _key: &str, _record: String) -> crate::Result<()> {
                Ok(())
            }
            async fn batch_set(&mut self, _records: HashMap<String, String>) -> crate::Result<()> {
                Ok(())
            }
            async fn remove(&mut self, _key: &str) -> crate::Result<()> {
                Ok(())
            }
        }

        let path = "./the-storage-path";
        super::set(path, None, Box::new(MyAdapter {})).await;
        let adapter = super::get(&PathBuf::from(path)).await.unwrap();
        let adapter = adapter.lock().await;
        assert_eq!(adapter.get("").await.unwrap(), "MY_ADAPTER_GET_RESPONSE".to_string());
    }

    #[test]
    fn parse_accounts_invalid() {
        let response = super::parse_accounts(&PathBuf::new(), &["{}".to_string()], &None);
        assert!(response.is_err());
    }

    async fn _create_account() -> (PathBuf, crate::account::AccountHandle) {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = crate::client::ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.testnet.chrysalis2.com")
            .unwrap()
            .build()
            .unwrap();
        let account_handle = manager
            .create_account(client_options)
            .unwrap()
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
        assert_eq!(parsed_account, &*account_handle.read().await);
    }
}

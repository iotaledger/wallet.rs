// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// RocksDB storage.
pub mod rocksdb;

/// Stronghold storage.
pub mod stronghold;

#[cfg(feature = "participation")]
use crate::address::AddressWrapper;
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
const KCV_KEY: &str = "iota-wallet-key-checksum_value";

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
pub struct MessageQueryFilter {
    message_type: Option<MessageType>,
}

impl MessageQueryFilter {
    pub fn message_type(message_type: Option<MessageType>) -> Self {
        Self { message_type }
    }
}

struct Storage {
    storage_path: PathBuf,
    inner: Box<dyn StorageAdapter + Sync + Send>,
    encryption_key: Option<[u8; 32]>,
}

impl Storage {
    async fn new(
        storage_path: PathBuf,
        storage_adapter: Box<dyn StorageAdapter + Send + Sync + 'static>,
        encryption_key: Option<[u8; 32]>,
    ) -> crate::Result<Self> {
        let storage_id = storage_adapter.id();
        let mut storage = Storage {
            storage_path,
            inner: storage_adapter,
            encryption_key: None,
        };

        if storage_id != stronghold::STORAGE_ID && encryption_key.is_some() {
            storage.set_encryption_key(encryption_key.unwrap()).await?;
        }

        Ok(storage)
    }

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

    async fn set_encryption_key(&mut self, encryption_key: [u8; 32]) -> crate::Result<()> {
        let record = key_checksum_value(&encryption_key)?;
        self.inner.set(KCV_KEY, serde_json::to_string(&record)?).await?;
        self.encryption_key.replace(encryption_key);
        Ok(())
    }

    async fn get_encryption_key_checksum(&self) -> crate::Result<Vec<u8>> {
        let record = self.inner.get(KCV_KEY).await?;
        Ok(serde_json::from_str(&record)?)
    }

    fn clear_encryption_key(&mut self) -> crate::Result<()> {
        self.encryption_key.take();
        Ok(())
    }

    #[cfg(test)]
    async fn remove_encryption_key_checksum(&mut self) -> crate::Result<()> {
        self.inner.remove(KCV_KEY).await
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

    pub(crate) fn is_encrypted(&self) -> bool {
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
        parse_accounts(&self.storage.storage_path, &accounts)
    }

    pub async fn save_account(&mut self, key: &str, account: &Account) -> crate::Result<()> {
        let index = AccountIndexation { key: key.to_string() };
        self.storage.set(key, account).await?;
        if !self.account_indexation.contains(&index) {
            init_account_dependency_index!(self, key, message_indexation);
            self.account_indexation.push(index);
        }
        // store it every time, because the password might changed
        self.storage
            .set(ACCOUNT_INDEXATION_KEY, &self.account_indexation)
            .await?;
        Ok(())
    }

    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    // used for ledger accounts to verify that the same menmonic is used for all accounts
    pub async fn save_first_ledger_address(
        &mut self,
        address: &iota_client::bee_message::address::Address,
    ) -> crate::Result<()> {
        self.storage.set("FIRST_LEDGER_ADDRESS", address).await?;
        Ok(())
    }

    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    pub async fn get_first_ledger_address(&self) -> crate::Result<iota_client::bee_message::address::Address> {
        let address: iota_client::bee_message::address::Address =
            serde_json::from_str(&self.storage.get("FIRST_LEDGER_ADDRESS").await?)?;
        Ok(address)
    }

    #[cfg(feature = "participation")]
    pub async fn save_participations(
        &mut self,
        account_index: usize,
        participations: Vec<crate::participation::types::Participation>,
    ) -> crate::Result<()> {
        self.storage
            .set(&format!("ACCOUNT-{}-PARTICIPATIONS", account_index), participations)
            .await?;
        Ok(())
    }

    #[cfg(feature = "participation")]
    pub async fn get_participations(
        &self,
        account_index: usize,
    ) -> crate::Result<Vec<crate::participation::types::Participation>> {
        let participations: Vec<crate::participation::types::Participation> = serde_json::from_str(
            &self
                .storage
                .get(&format!("ACCOUNT-{}-PARTICIPATIONS", account_index))
                .await?,
        )?;
        Ok(participations)
    }

    #[cfg(feature = "participation")]
    pub async fn save_participation_address(
        &mut self,
        account_index: usize,
        participation_address: AddressWrapper,
    ) -> crate::Result<()> {
        self.storage
            .set(
                &format!("ACCOUNT-{}-PARTICIPATIONADDRESS", account_index),
                participation_address,
            )
            .await?;
        Ok(())
    }

    #[cfg(feature = "participation")]
    pub async fn get_participation_address(&self, account_index: usize) -> crate::Result<AddressWrapper> {
        let participation_address: AddressWrapper = serde_json::from_str(
            &self
                .storage
                .get(&format!("ACCOUNT-{}-PARTICIPATIONADDRESS", account_index))
                .await?,
        )?;
        Ok(participation_address)
    }

    #[cfg(feature = "participation")]
    pub async fn get_participation_outputs(
        &self,
        account_index: usize,
    ) -> crate::Result<crate::participation::types::OutputStatusResponses> {
        let participation_outputs: crate::participation::types::OutputStatusResponses = serde_json::from_str(
            &self
                .storage
                .get(&format!("ACCOUNT-{}-PARTICIPATION-OUTPUTS", account_index))
                .await?,
        )?;
        Ok(participation_outputs)
    }

    #[cfg(feature = "participation")]
    pub async fn save_participation_outputs(
        &mut self,
        account_index: usize,
        participation_outputs: crate::participation::types::OutputStatusResponses,
    ) -> crate::Result<()> {
        self.storage
            .set(
                &format!("ACCOUNT-{}-PARTICIPATION-OUTPUTS", account_index),
                participation_outputs,
            )
            .await?;
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
        filter: &MessageQueryFilter,
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
        filter: MessageQueryFilter,
    ) -> crate::Result<Vec<Message>> {
        let filtered_message_indexation = self.query_message_indexation(account, &filter)?;

        let mut messages = Vec::new();
        let iter = filtered_message_indexation.into_iter().skip(skip);
        for index in if count == 0 {
            iter.collect::<Vec<&MessageIndexation>>()
        } else {
            iter.take(count).collect::<Vec<&MessageIndexation>>()
        } {
            if let Ok(mut message) = serde_json::from_str::<Message>(&self.get(&index.key.to_string()).await?) {
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

fn key_checksum_value(encryption_key: &[u8; 32]) -> crate::Result<Vec<u8>> {
    let mut nonce = [0u8; XChaCha20Poly1305::NONCE_LENGTH];
    crypto::utils::rand::fill(&mut nonce).map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;
    key_checksum_value_with_nonce(encryption_key, nonce)
}

fn key_checksum_value_with_nonce(
    encryption_key: &[u8; 32],
    nonce: [u8; XChaCha20Poly1305::NONCE_LENGTH],
) -> crate::Result<Vec<u8>> {
    let mut tag = vec![0u8; XChaCha20Poly1305::TAG_LENGTH];
    let data = [0u8; XChaCha20Poly1305::KEY_LENGTH];

    let mut ciphertext = [0u8; XChaCha20Poly1305::KEY_LENGTH];
    // we can unwrap here since we know the lengths are valid
    XChaCha20Poly1305::encrypt(
        encryption_key.try_into().unwrap(),
        &nonce.try_into().unwrap(),
        &[],
        &data,
        &mut ciphertext,
        tag.as_mut_slice().try_into().unwrap(),
    )
    .map_err(|e| crate::Error::RecordEncrypt(format!("{:?}", e)))?;

    let mut data = Vec::from(nonce);
    data.extend_from_slice(&ciphertext[0..3]);

    Ok(data)
}

fn split_key_checksum(record: &[u8]) -> crate::Result<([u8; XChaCha20Poly1305::NONCE_LENGTH], Vec<u8>)> {
    let mut record = record;

    let mut nonce = [0u8; XChaCha20Poly1305::NONCE_LENGTH];
    let mut kcv = Vec::new();

    record.read_exact(&mut nonce)?;
    record.read_to_end(&mut kcv)?;

    Ok((nonce, kcv))
}

pub(crate) async fn is_key_valid(storage_path: &Path, encryption_key: &[u8; 32]) -> crate::Result<bool> {
    let record = crate::storage::get_encryption_key_checksum(storage_path).await;

    match record {
        Ok(record) => {
            let (nonce, _kcv) = split_key_checksum(&record)?;
            if record == key_checksum_value_with_nonce(encryption_key, nonce)? {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(crate::Error::RecordNotFound) => {
            let storage_handle = crate::storage::get(storage_path).await?;
            let is_valid = match storage_handle.lock().await.get(ACCOUNT_INDEXATION_KEY).await {
                // Existing DB
                Ok(indexation) => match serde_json::from_str::<Vec<AccountIndexation>>(&indexation) {
                    Ok(_account_indexation) => {
                        // DB is not encrypted, or is it possible that someone already set the correct password?
                        Ok(true)
                    }
                    Err(_) => match decrypt_record(&indexation, encryption_key) {
                        Ok(indexation) => Ok(serde_json::from_str::<Vec<AccountIndexation>>(&indexation).is_ok()),
                        Err(_) => Ok(false),
                    },
                },
                // Newly created DB
                Err(crate::Error::RecordNotFound) => Ok(true),
                // Some other error
                Err(e) => Err(e),
            };
            is_valid
        }
        Err(e) => Err(e),
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
) -> crate::Result<()> {
    let mut instances = INSTANCES.get_or_init(Default::default).write().await;
    #[allow(unused_variables)]
    let storage_id = storage.id();
    let storage = Storage::new(storage_path.as_ref().to_path_buf(), storage, encryption_key).await?;

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

    Ok(())
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
        storage_manager.storage.set_encryption_key(encryption_key).await
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.to_path_buf()))
    }
}

pub(crate) async fn get_encryption_key_checksum(storage_path: &Path) -> crate::Result<Vec<u8>> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
    if let Some(instance) = instances.get(storage_path) {
        let storage_manager = instance.lock().await;
        storage_manager.storage.get_encryption_key_checksum().await
    } else {
        Err(crate::Error::StorageAdapterNotSet(storage_path.to_path_buf()))
    }
}

pub(crate) async fn clear_encryption_key(storage_path: &Path) -> crate::Result<()> {
    let instances = INSTANCES.get_or_init(Default::default).read().await;
    if let Some(instance) = instances.get(storage_path) {
        let mut storage_manager = instance.lock().await;
        storage_manager.storage.clear_encryption_key()
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
    let record: Vec<u8> = serde_json::from_str(record)?;
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

fn parse_accounts(storage_path: &Path, accounts: &[String]) -> crate::Result<Vec<Account>> {
    let mut parsed_accounts: Vec<Account> = Vec::new();
    for account in accounts {
        let account_json = if account.starts_with('{') {
            Some(account.to_string())
        } else {
            None
        };
        if let Some(json) = account_json {
            let mut acc = serde_json::from_str::<Account>(&json)?;
            acc.set_storage_path(storage_path.to_path_buf());
            parsed_accounts.push(acc);
        } else {
            return Err(crate::Error::StorageIsEncrypted);
        }
    }
    Ok(parsed_accounts)
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
        super::set(path, None, Box::new(MyAdapter {})).await.unwrap();
        let adapter = super::get(&PathBuf::from(path)).await.unwrap();
        let adapter = adapter.lock().await;
        assert_eq!(adapter.get("").await.unwrap(), "MY_ADAPTER_GET_RESPONSE".to_string());
    }

    #[test]
    fn parse_accounts_invalid() {
        let response = super::parse_accounts(&PathBuf::new(), &["{}".to_string()]);
        assert!(response.is_err());
    }

    async fn _create_account() -> (PathBuf, crate::account::AccountHandle) {
        let manager = crate::test_utils::get_account_manager().await;

        let client_options = crate::client::ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
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
        );
        assert!(response.is_ok());
        let parsed_accounts = response.unwrap();
        let parsed_account = parsed_accounts.first().unwrap();
        assert_eq!(parsed_account, &*account_handle.read().await);
    }

    #[tokio::test]
    async fn remove_encryption_key_checksum() {
        let manager = crate::test_utils::get_account_manager().await;
        manager.set_storage_password("password").await.unwrap();

        let storage_handle = crate::storage::get(manager.storage_path()).await.unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .get_encryption_key_checksum()
            .await
            .unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .remove_encryption_key_checksum()
            .await
            .unwrap();
        match storage_handle
            .lock()
            .await
            .storage
            .get_encryption_key_checksum()
            .await
            .err()
            .unwrap()
        {
            crate::Error::RecordNotFound => {}
            e => panic!("{:?}", e),
        };
    }

    #[tokio::test]
    async fn wrong_storage_password_without_kcv() {
        let manager = crate::test_utils::get_account_manager().await;
        manager.set_storage_password("password").await.unwrap();

        let _ = crate::test_utils::AccountCreator::new(&manager).create().await;
        assert_eq!(manager.get_accounts().await.unwrap().len(), 1);

        manager.clear_storage_password().await.unwrap();
        let storage_handle = crate::storage::get(manager.storage_path()).await.unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .remove_encryption_key_checksum()
            .await
            .unwrap();

        match manager.set_storage_password("wrong-password").await.err().unwrap() {
            crate::Error::RecordDecrypt(_) => {}
            e => panic!("{:?}", e),
        }
    }

    #[tokio::test]
    async fn correct_storage_password_without_kcv() {
        let manager = crate::test_utils::get_account_manager().await;
        manager.set_storage_password("password").await.unwrap();

        let _ = crate::test_utils::AccountCreator::new(&manager).create().await;
        assert_eq!(manager.get_accounts().await.unwrap().len(), 1);

        manager.clear_storage_password().await.unwrap();
        let storage_handle = crate::storage::get(manager.storage_path()).await.unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .remove_encryption_key_checksum()
            .await
            .unwrap();

        manager.set_storage_password("password").await.unwrap();
    }

    #[tokio::test]
    async fn wrong_storage_password_without_kcv_and_all_accounts_deleted() {
        let manager = crate::test_utils::get_account_manager().await;
        manager.set_storage_password("password").await.unwrap();

        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        assert_eq!(manager.get_accounts().await.unwrap().len(), 1);

        manager
            .remove_account(account_handle.read().await.id())
            .await
            .expect("failed to remove account");
        manager.clear_storage_password().await.unwrap();
        let storage_handle = crate::storage::get(manager.storage_path()).await.unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .remove_encryption_key_checksum()
            .await
            .unwrap();

        match manager.set_storage_password("wrong-password").await.err().unwrap() {
            crate::Error::RecordDecrypt(_) => {}
            e => panic!("{:?}", e),
        }
    }

    #[tokio::test]
    async fn correct_storage_password_without_kcv_and_all_accounts_deleted() {
        let manager = crate::test_utils::get_account_manager().await;
        manager.set_storage_password("password").await.unwrap();

        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        assert_eq!(manager.get_accounts().await.unwrap().len(), 1);

        manager
            .remove_account(account_handle.read().await.id())
            .await
            .expect("failed to remove account");
        manager.clear_storage_password().await.unwrap();
        let storage_handle = crate::storage::get(manager.storage_path()).await.unwrap();
        storage_handle
            .lock()
            .await
            .storage
            .remove_encryption_key_checksum()
            .await
            .unwrap();

        manager.set_storage_password("password").await.unwrap();
    }
}

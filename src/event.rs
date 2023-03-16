// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    address::AddressWrapper,
    message::{Message, MessageId},
};

use getset::Getters;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use std::{
    ops::Deref,
    sync::{Arc, Mutex as StdMutex},
};

/// The event identifier type.
pub type EventId = [u8; 32];

fn generate_indexation_id() -> String {
    let mut key = [0; 32];
    crypto::utils::rand::fill(&mut key).unwrap();
    hex::encode(key)
}

/// The balance change event payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct BalanceChange {
    /// The change amount if it was a spent event.
    pub spent: u64,
    /// The change amount if it was a receive event.
    pub received: u64,
}

impl BalanceChange {
    pub(crate) fn spent(value: u64) -> Self {
        Self {
            spent: value,
            received: 0,
        }
    }

    pub(crate) fn received(value: u64) -> Self {
        Self {
            spent: 0,
            received: value,
        }
    }
}

/// The balance change event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct BalanceEvent {
    /// Event unique identifier.
    #[serde(rename = "indexationId")]
    pub indexation_id: String,
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// The associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
    /// The message id associated with the balance change.
    /// Note that this might be `None` without
    /// [AccountManagerBuilder#with_sync_spent_outputs](struct.AccountManagerBuilder.html#method.
    /// with_sync_spent_outputs).
    #[serde(rename = "messageId")]
    pub message_id: Option<MessageId>,
    /// Whether the event is associated with a remainder output or not.
    /// Note that this might be `None` if we couldn't get the message object from the node.
    remainder: Option<bool>,
    /// The balance change data.
    #[serde(rename = "balanceChange")]
    pub balance_change: BalanceChange,
}

/// The `address consolidation needed` data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AddressConsolidationNeeded {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// The associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
}

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
/// Ledger generate address event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct LedgerAddressGeneration {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The transfer event type.
    pub event: AddressData,
}

/// A transaction-related event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionEvent {
    /// Event unique identifier.
    #[serde(rename = "indexationId")]
    pub indexation_id: String,
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The event message.
    pub message: Message,
}

/// A transaction confirmation state change event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionConfirmationChangeEvent {
    /// Event unique identifier.
    #[serde(rename = "indexationId")]
    pub indexation_id: String,
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The event message.
    pub message: Message,
    /// The confirmed state of the transaction.
    pub confirmed: bool,
}

/// Transaction reattachment event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionReattachmentEvent {
    /// Event unique identifier.
    #[serde(rename = "indexationId")]
    pub indexation_id: String,
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The event message.
    pub message: Message,
    /// The id of the message that was reattached.
    #[serde(rename = "reattachedMessageId")]
    pub reattached_message_id: MessageId,
}

/// Address data for TransferProgressType::GeneratingRemainderDepositAddress and LedgerAddressGeneration.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    pub address: String,
}

/// Prepared transaction event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct PreparedTransactionData {
    /// Transaction inputs.
    pub inputs: Vec<TransactionIO>,
    /// Transaction outputs.
    pub outputs: Vec<TransactionIO>,
    /// The indexation data.
    pub data: Option<String>,
}

/// Input or output data for PreparedTransactionData
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionIO {
    /// Address
    pub address: String,
    /// Amount
    pub amount: u64,
    /// Remainder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remainder: Option<bool>,
}

/// Transfer event type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransferProgressType {
    /// Syncing account.
    SyncingAccount,
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(PreparedTransactionData),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPoW,
    /// Broadcasting.
    Broadcasting,
}

/// Transfer event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransferProgress {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The transfer event type.
    pub event: TransferProgressType,
}

/// Migration event type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MigrationProgressType {
    /// Fetching migration data on the given address range.
    FetchingMigrationData {
        /// The initial address index on the fetch range.
        #[serde(rename = "initialAddresIndex")]
        initial_address_index: u64,
        /// The final address index on the fetch range.
        #[serde(rename = "finalAddressIndex")]
        final_address_index: u64,
    },
    /// Mining the bundle with the given spent address.
    MiningBundle {
        /// The spent address.
        address: String,
    },
    /// Signing the bundle.
    SigningBundle {
        /// The addresses associated with the bundle.
        addresses: Vec<String>,
    },
    /// Broadcasting the given bundle hash.
    BroadcastingBundle {
        /// The bundle hash.
        #[serde(rename = "bundleHash")]
        bundle_hash: String,
    },
    /// Transaction confirmation event.
    TransactionConfirmed {
        /// The bundle hash.
        #[serde(rename = "bundleHash")]
        bundle_hash: String,
    },
}

/// Migration event data.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct MigrationProgress {
    /// The transfer event type.
    pub event: MigrationProgressType,
}

trait EventHandler {
    fn id(&self) -> &EventId;
}

macro_rules! event_handler_impl {
    ($ty:ident) => {
        impl EventHandler for $ty {
            fn id(&self) -> &EventId {
                &self.id
            }
        }
    };
}

struct BalanceEventHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&BalanceEvent) + Send>,
}

event_handler_impl!(BalanceEventHandler);

struct ErrorHandler {
    id: EventId,
    /// The on error callback.
    on_error: Box<dyn Fn(&crate::Error) + Send>,
}

event_handler_impl!(ErrorHandler);

#[derive(PartialEq)]
pub(crate) enum TransactionEventType {
    NewTransaction,
    Broadcast,
}

struct TransactionEventHandler {
    id: EventId,
    event_type: TransactionEventType,
    /// The on event callback.
    on_event: Box<dyn Fn(&TransactionEvent) + Send>,
}

event_handler_impl!(TransactionEventHandler);

struct TransactionConfirmationChangeEventHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&TransactionConfirmationChangeEvent) + Send>,
}

event_handler_impl!(TransactionConfirmationChangeEventHandler);

struct TransactionReattachmentEventHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&TransactionReattachmentEvent) + Send>,
}

event_handler_impl!(TransactionReattachmentEventHandler);

#[cfg(feature = "stronghold")]
struct StrongholdStatusChangeEventHandler {
    id: EventId,
    on_event: Box<dyn Fn(&crate::StrongholdStatus) + Send>,
}

#[cfg(feature = "stronghold")]
event_handler_impl!(StrongholdStatusChangeEventHandler);

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
struct AddressConsolidationNeededHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&AddressConsolidationNeeded) + Send>,
}

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
event_handler_impl!(AddressConsolidationNeededHandler);

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
struct LedgerAddressGenerationHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&LedgerAddressGeneration) + Send>,
}

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
event_handler_impl!(LedgerAddressGenerationHandler);

struct TransferProgressHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&TransferProgress) + Send>,
}

event_handler_impl!(TransferProgressHandler);

struct MigrationProgressHandler {
    id: EventId,
    /// The on event callback.
    on_event: Box<dyn Fn(&MigrationProgress) + Send>,
}

event_handler_impl!(MigrationProgressHandler);

type BalanceListeners = Arc<Mutex<Vec<BalanceEventHandler>>>;
type TransactionListeners = Arc<Mutex<Vec<TransactionEventHandler>>>;
type TransactionConfirmationChangeListeners = Arc<Mutex<Vec<TransactionConfirmationChangeEventHandler>>>;
type TransactionReattachmentListeners = Arc<Mutex<Vec<TransactionReattachmentEventHandler>>>;
type ErrorListeners = Arc<StdMutex<Vec<ErrorHandler>>>;
#[cfg(feature = "stronghold")]
type StrongholdStatusChangeListeners = Arc<Mutex<Vec<StrongholdStatusChangeEventHandler>>>;
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
type AddressConsolidationNeededListeners = Arc<Mutex<Vec<AddressConsolidationNeededHandler>>>;
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
type LedgerAddressGenerationListeners = Arc<Mutex<Vec<LedgerAddressGenerationHandler>>>;
type TransferProgressListeners = Arc<Mutex<Vec<TransferProgressHandler>>>;
type MigrationProgressListeners = Arc<Mutex<Vec<MigrationProgressHandler>>>;

fn generate_event_id() -> EventId {
    let mut id = [0; 32];
    crypto::utils::rand::fill(&mut id).unwrap();
    id
}

async fn remove_event_listener<T: EventHandler>(id: &EventId, listeners: &Arc<Mutex<Vec<T>>>) {
    let mut listeners = listeners.lock().await;
    if let Some(position) = listeners.iter().position(|e| e.id() == id) {
        listeners.remove(position);
    }
}

/// Gets the balance change listeners array.
fn balance_listeners() -> &'static BalanceListeners {
    static LISTENERS: Lazy<BalanceListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the transaction listeners array.
fn transaction_listeners() -> &'static TransactionListeners {
    static LISTENERS: Lazy<TransactionListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the transaction confirmation change listeners array.
fn transaction_confirmation_change_listeners() -> &'static TransactionConfirmationChangeListeners {
    static LISTENERS: Lazy<TransactionConfirmationChangeListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the transaction reattachment listeners array.
fn transaction_reattachment_listeners() -> &'static TransactionReattachmentListeners {
    static LISTENERS: Lazy<TransactionReattachmentListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the balance change listeners array.
fn error_listeners() -> &'static ErrorListeners {
    static LISTENERS: Lazy<ErrorListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the stronghold status change listeners array.
#[cfg(feature = "stronghold")]
fn stronghold_status_change_listeners() -> &'static StrongholdStatusChangeListeners {
    static LISTENERS: Lazy<StrongholdStatusChangeListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the address consolodation needed listeners array.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
fn address_consolidation_needed_listeners() -> &'static AddressConsolidationNeededListeners {
    static LISTENERS: Lazy<AddressConsolidationNeededListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets address that will be generated with a prompt on the ledger.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
fn ledger_address_generation_listeners() -> &'static LedgerAddressGenerationListeners {
    static LISTENERS: Lazy<LedgerAddressGenerationListeners> = Lazy::new(Default::default);
    &LISTENERS
}

fn transfer_progress_listeners() -> &'static TransferProgressListeners {
    static LISTENERS: Lazy<TransferProgressListeners> = Lazy::new(Default::default);
    &LISTENERS
}

fn migration_progress_listeners() -> &'static MigrationProgressListeners {
    static LISTENERS: Lazy<MigrationProgressListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Listen to balance changes.
pub async fn on_balance_change<F: Fn(&BalanceEvent) + Send + 'static>(cb: F) -> EventId {
    let mut l = balance_listeners().lock().await;
    let id = generate_event_id();
    l.push(BalanceEventHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Removes the balance change listener associated with the given identifier.
pub async fn remove_balance_change_listener(id: &EventId) {
    remove_event_listener(id, balance_listeners()).await;
}

/// Emits a balance change event.
pub(crate) async fn emit_balance_change(
    account: &Account,
    address: &AddressWrapper,
    message_id: Option<MessageId>,
    balance_change: BalanceChange,
    persist: bool,
) -> crate::Result<()> {
    let listeners = balance_listeners().lock().await;
    let remainder = if balance_change.spent > 0 {
        Some(false)
    } else {
        match message_id {
            Some(id) => account
                .get_message(&id)
                .await
                .and_then(|message| message.is_remainder(address)),
            None => None,
        }
    };
    let event = BalanceEvent {
        indexation_id: generate_indexation_id(),
        account_id: account.id().to_string(),
        address: address.clone(),
        message_id,
        remainder,
        balance_change,
    };

    if persist {
        crate::storage::get(account.storage_path())
            .await?
            .lock()
            .await
            .save_balance_change_event(&event)
            .await?;
    }

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }

    Ok(())
}

/// Emits a transaction-related event.
pub(crate) async fn emit_transaction_event(
    event_type: TransactionEventType,
    account: &Account,
    message: Message,
    persist: bool,
) -> crate::Result<()> {
    let listeners = transaction_listeners().lock().await;
    let event = TransactionEvent {
        indexation_id: generate_indexation_id(),
        account_id: account.id().to_string(),
        message,
    };

    if persist {
        let storage_handle = crate::storage::get(account.storage_path()).await?;
        let mut storage = storage_handle.lock().await;
        match event_type {
            TransactionEventType::Broadcast => {
                storage.save_broadcast_event(&event).await?;
            }
            TransactionEventType::NewTransaction => {
                storage.save_new_transaction_event(&event).await?;
            }
        }
    }

    for listener in listeners.deref() {
        if listener.event_type == event_type {
            (listener.on_event)(&event);
        }
    }

    Ok(())
}

/// Emits a transaction confirmation state change event.
pub(crate) async fn emit_confirmation_state_change(
    account: &Account,
    message: Message,
    confirmed: bool,
    persist: bool,
) -> crate::Result<()> {
    let listeners = transaction_confirmation_change_listeners().lock().await;
    let event = TransactionConfirmationChangeEvent {
        indexation_id: generate_indexation_id(),
        account_id: account.id().to_string(),
        message,
        confirmed,
    };

    if persist {
        crate::storage::get(account.storage_path())
            .await?
            .lock()
            .await
            .save_transaction_confirmation_event(&event)
            .await?;
    }

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }

    Ok(())
}

/// Emits a transaction reattachment change event.
pub(crate) async fn emit_reattachment_event(
    account: &Account,
    reattached_message_id: MessageId,
    message: &Message,
    persist: bool,
) -> crate::Result<()> {
    let listeners = transaction_reattachment_listeners().lock().await;
    let event = TransactionReattachmentEvent {
        indexation_id: generate_indexation_id(),
        account_id: account.id().to_string(),
        message: message.clone(),
        reattached_message_id,
    };

    if persist {
        crate::storage::get(account.storage_path())
            .await?
            .lock()
            .await
            .save_reattachment_event(&event)
            .await?;
    }

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }

    Ok(())
}

/// Adds a transaction-related event listener.
async fn add_transaction_listener<F: Fn(&TransactionEvent) + Send + 'static>(
    event_type: TransactionEventType,
    cb: F,
) -> EventId {
    let mut l = transaction_listeners().lock().await;
    let id = generate_event_id();
    l.push(TransactionEventHandler {
        id,
        event_type,
        on_event: Box::new(cb),
    });
    id
}

/// Listen to new messages.
pub async fn on_new_transaction<F: Fn(&TransactionEvent) + Send + 'static>(cb: F) -> EventId {
    add_transaction_listener(TransactionEventType::NewTransaction, cb).await
}

/// Removes the new transaction listener associated with the given identifier.
pub async fn remove_new_transaction_listener(id: &EventId) {
    remove_event_listener(id, transaction_listeners()).await;
}

/// Listen to transaction confirmation state change.
pub async fn on_confirmation_state_change<F: Fn(&TransactionConfirmationChangeEvent) + Send + 'static>(
    cb: F,
) -> EventId {
    let mut l = transaction_confirmation_change_listeners().lock().await;
    let id = generate_event_id();
    l.push(TransactionConfirmationChangeEventHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Listen to transaction reattachment change.
pub async fn on_reattachment<F: Fn(&TransactionReattachmentEvent) + Send + 'static>(cb: F) -> EventId {
    let mut l = transaction_reattachment_listeners().lock().await;
    let id = generate_event_id();
    l.push(TransactionReattachmentEventHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Removes the new confirmation state change listener associated with the given identifier.
pub async fn remove_confirmation_state_change_listener(id: &EventId) {
    remove_event_listener(id, transaction_confirmation_change_listeners()).await;
}

/// Removes the reattachment listener associated with the given identifier.
pub async fn remove_reattachment_listener(id: &EventId) {
    remove_event_listener(id, transaction_reattachment_listeners()).await;
}

/// Listen to transaction broadcast.
pub async fn on_broadcast<F: Fn(&TransactionEvent) + Send + 'static>(cb: F) -> EventId {
    add_transaction_listener(TransactionEventType::Broadcast, cb).await
}

/// Removes the broadcast listener associated with the given identifier.
pub async fn remove_broadcast_listener(id: &EventId) {
    remove_event_listener(id, transaction_listeners()).await;
}

pub(crate) fn emit_error(error: &crate::Error) {
    let listeners = error_listeners().lock().unwrap();
    for listener in listeners.deref() {
        (listener.on_error)(error)
    }
}

/// Listen to errors.
pub fn on_error<F: Fn(&crate::Error) + Send + 'static>(cb: F) -> EventId {
    let mut l = error_listeners().lock().unwrap();
    let id = generate_event_id();
    l.push(ErrorHandler {
        id,
        on_error: Box::new(cb),
    });
    id
}

/// Removes the error listener associated with the given identifier.
pub fn remove_error_listener(id: &EventId) {
    let mut listeners = error_listeners().lock().unwrap();
    if let Some(position) = listeners.iter().position(|e| e.id() == id) {
        listeners.remove(position);
    }
}

#[cfg(feature = "stronghold")]
pub(crate) async fn emit_stronghold_status_change(status: &crate::StrongholdStatus) {
    let listeners = stronghold_status_change_listeners().lock().await;
    for listener in listeners.deref() {
        (listener.on_event)(status)
    }
}

/// Listen to stronghold status change events.
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub async fn on_stronghold_status_change<F: Fn(&crate::StrongholdStatus) + Send + 'static>(cb: F) -> EventId {
    let mut l = stronghold_status_change_listeners().lock().await;
    let id = generate_event_id();
    l.push(StrongholdStatusChangeEventHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Removes the stronghold status change listener associated with the given identifier.
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub async fn remove_stronghold_status_change_listener(id: &EventId) {
    remove_event_listener(id, stronghold_status_change_listeners()).await;
}

/// Listen to `address consolidation needed` events.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub async fn on_address_consolidation_needed<F: Fn(&AddressConsolidationNeeded) + Send + 'static>(cb: F) -> EventId {
    let mut l = address_consolidation_needed_listeners().lock().await;
    let id = generate_event_id();
    l.push(AddressConsolidationNeededHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Removes the balance change listener associated with the given identifier.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub async fn remove_address_consolidation_needed_listener(id: &EventId) {
    remove_event_listener(id, address_consolidation_needed_listeners()).await;
}

/// Emits a balance change event.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
pub(crate) async fn emit_address_consolidation_needed(account: &Account, address: AddressWrapper) {
    let listeners = address_consolidation_needed_listeners().lock().await;
    let event = AddressConsolidationNeeded {
        account_id: account.id().to_string(),
        address,
    };

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

/// Listen to `ledger address generation` events.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub async fn on_ledger_address_generation<F: Fn(&LedgerAddressGeneration) + Send + 'static>(cb: F) -> EventId {
    let mut listeners = ledger_address_generation_listeners().lock().await;
    let id = generate_event_id();
    listeners.push(LedgerAddressGenerationHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Removes the balance change listener associated with the given identifier.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub async fn remove_ledger_address_generation_listener(id: &EventId) {
    remove_event_listener(id, ledger_address_generation_listeners()).await;
}

/// Emits a balance change event.
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
pub(crate) async fn emit_ledger_address_generation(account: &Account, address: String) {
    let listeners = ledger_address_generation_listeners().lock().await;
    let event = LedgerAddressGeneration {
        account_id: account.id().to_string(),
        event: AddressData { address },
    };

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

/// Listen to a transfer event.
pub async fn on_transfer_progress<F: Fn(&TransferProgress) + Send + 'static>(cb: F) -> EventId {
    let mut l = transfer_progress_listeners().lock().await;
    let id = generate_event_id();
    l.push(TransferProgressHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Remove a transfer event listener.
pub async fn remove_transfer_progress_listener(id: &EventId) {
    remove_event_listener(id, transfer_progress_listeners()).await;
}

/// Emit a transfer event.
pub(crate) async fn emit_transfer_progress(account_id: String, event: TransferProgressType) {
    let listeners = transfer_progress_listeners().lock().await;
    let event = TransferProgress { account_id, event };

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

/// Listen to a migration event.
pub async fn on_migration_progress<F: Fn(&MigrationProgress) + Send + 'static>(cb: F) -> EventId {
    let mut l = migration_progress_listeners().lock().await;
    let id = generate_event_id();
    l.push(MigrationProgressHandler {
        id,
        on_event: Box::new(cb),
    });
    id
}

/// Remove a migration event listener.
pub async fn remove_migration_progress_listener(id: &EventId) {
    remove_event_listener(id, migration_progress_listeners()).await;
}

/// Emit a migration event.
pub(crate) async fn emit_migration_progress(event: MigrationProgressType) {
    let listeners = migration_progress_listeners().lock().await;
    let event = MigrationProgress { event };

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_fork::rusty_fork_test;

    fn _create_and_drop_error() {
        let _ = crate::Error::RecordNotFound;
    }

    // have to fork this test so other errors dropped doesn't affect it
    rusty_fork_test! {
        #[test]
        fn error_events() {
            on_error(|error| {
                assert!(matches!(error, crate::Error::RecordNotFound));
            });
            _create_and_drop_error();
        }
    }

    rusty_fork_test! {
        #[test]
        fn balance_events() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let manager = crate::test_utils::get_account_manager().await;
                let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                let account = account_handle.read().await;
                let account_id = account.id().to_string();
                on_balance_change(move |event| {
                    assert!(event.account_id == account_id);
                    assert!(event.balance_change.spent == 5);
                    assert!(event.balance_change.received == 0);
                })
                .await;

                emit_balance_change(
                    &account,
                    &crate::test_utils::generate_random_iota_address(),
                    None,
                    BalanceChange::spent(5),
                    true,
                )
                .await
                .unwrap();
            });
        }

        #[test]
        fn on_new_transaction_event() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let manager = crate::test_utils::get_account_manager().await;
                let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                let account = account_handle.read().await;
                let account_id = account.id().to_string();
                let message = crate::test_utils::GenerateMessageBuilder::default().build().await;
                let message_ = message.clone();

                on_new_transaction(move |event| {
                    assert!(event.account_id == account_id);
                    assert!(event.message == message_);
                })
                .await;

                emit_transaction_event(TransactionEventType::NewTransaction, &account, message, true)
                    .await
                    .unwrap();
            });
        }

        #[test]
        fn on_reattachment_event() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let manager = crate::test_utils::get_account_manager().await;
                let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                let account = account_handle.read().await;
                let account_id = account.id().to_string();
                let message = crate::test_utils::GenerateMessageBuilder::default().build().await;
                let message_ = message.clone();

                on_reattachment(move |event| {
                    assert!(event.account_id == account_id);
                    assert!(event.message == message_);
                })
                .await;

                emit_reattachment_event(&account, *message.id(), &message, true)
                    .await
                    .unwrap();
            });
        }

        #[test]
        fn on_broadcast_event() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let manager = crate::test_utils::get_account_manager().await;
                let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                let account = account_handle.read().await;
                let account_id = account.id().to_string();
                let message = crate::test_utils::GenerateMessageBuilder::default().build().await;
                let message_ = message.clone();

                on_broadcast(move |event| {
                    assert!(event.account_id == account_id);
                    assert!(event.message == message_);
                })
                .await;

                emit_transaction_event(TransactionEventType::Broadcast, &account, message, true)
                    .await
                    .unwrap();
            });
        }

        #[test]
        fn on_confirmation_state_change_event() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let manager = crate::test_utils::get_account_manager().await;
                let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
                let account = account_handle.read().await;
                let account_id = account.id().to_string();
                let message = crate::test_utils::GenerateMessageBuilder::default().build().await;
                let message_ = message.clone();
                let confirmed = true;

                on_confirmation_state_change(move |event| {
                    assert!(event.account_id == account_id);
                    assert!(event.message == message_);
                    assert!(event.confirmed == confirmed);
                })
                .await;

                emit_confirmation_state_change(&account, message.clone(), confirmed, true)
                    .await
                    .unwrap();
            });
        }
    }
}

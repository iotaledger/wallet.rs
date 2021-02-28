// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, address::AddressWrapper, message::Message};

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

/// The balance change event payload.
#[derive(Getters, Serialize, Deserialize)]
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
#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct BalanceEvent {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// The associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
    /// The balance change data.
    #[serde(rename = "balanceChange")]
    pub balance_change: BalanceChange,
}

/// A transaction-related event data.
#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionEvent {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The event message.
    pub message: Message,
}

impl TransactionEvent {
    #[doc(hidden)]
    pub fn cloned_message(&self) -> Message {
        self.message.clone()
    }
}

/// A transaction-related event data.
#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TransactionConfirmationChangeEvent {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    pub account_id: String,
    /// The event message.
    pub message: Message,
    /// The confirmed state of the transaction.
    pub confirmed: bool,
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
    Reattachment,
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

#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
struct StrongholdStatusChangeEventHandler {
    id: EventId,
    on_event: Box<dyn Fn(&crate::StrongholdStatus) + Send>,
}

#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
event_handler_impl!(StrongholdStatusChangeEventHandler);

type BalanceListeners = Arc<Mutex<Vec<BalanceEventHandler>>>;
type TransactionListeners = Arc<Mutex<Vec<TransactionEventHandler>>>;
type TransactionConfirmationChangeListeners = Arc<Mutex<Vec<TransactionConfirmationChangeEventHandler>>>;
type ErrorListeners = Arc<StdMutex<Vec<ErrorHandler>>>;
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
type StrongholdStatusChangeListeners = Arc<Mutex<Vec<StrongholdStatusChangeEventHandler>>>;

fn generate_event_id() -> EventId {
    let mut id = [0; 32];
    crypto::rand::fill(&mut id).unwrap();
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

/// Gets the balance change listeners array.
fn error_listeners() -> &'static ErrorListeners {
    static LISTENERS: Lazy<ErrorListeners> = Lazy::new(Default::default);
    &LISTENERS
}

/// Gets the stronghold status change listeners array.
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
fn stronghold_status_change_listeners() -> &'static StrongholdStatusChangeListeners {
    static LISTENERS: Lazy<StrongholdStatusChangeListeners> = Lazy::new(Default::default);
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
    balance_change: BalanceChange,
) -> crate::Result<()> {
    let listeners = balance_listeners().lock().await;
    let event = BalanceEvent {
        account_id: account.id().to_string(),
        address: address.clone(),
        balance_change,
    };

    crate::storage::get(account.storage_path())
        .await?
        .lock()
        .await
        .save_balance_change_event(&event)
        .await?;

    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }

    Ok(())
}

/// Emits a transaction-related event.
pub(crate) async fn emit_transaction_event(event_type: TransactionEventType, account_id: String, message: &Message) {
    let listeners = transaction_listeners().lock().await;
    let event = TransactionEvent {
        account_id,
        message: message.clone(),
    };
    for listener in listeners.deref() {
        if listener.event_type == event_type {
            (listener.on_event)(&event);
        }
    }
}

/// Emits a transaction confirmation state change event.
pub(crate) async fn emit_confirmation_state_change(account_id: String, message: &Message, confirmed: bool) {
    let listeners = transaction_confirmation_change_listeners().lock().await;
    let event = TransactionConfirmationChangeEvent {
        account_id,
        message: message.clone(),
        confirmed,
    };
    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
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

/// Removes the new confirmation state change listener associated with the given identifier.
pub async fn remove_confirmation_state_change_listener(id: &EventId) {
    remove_event_listener(id, transaction_confirmation_change_listeners()).await;
}

/// Listen to transaction reattachment.
pub async fn on_reattachment<F: Fn(&TransactionEvent) + Send + 'static>(cb: F) -> EventId {
    add_transaction_listener(TransactionEventType::Reattachment, cb).await
}

/// Removes the reattachment listener associated with the given identifier.
pub async fn remove_reattachment_listener(id: &EventId) {
    remove_event_listener(id, transaction_listeners()).await;
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
        (listener.on_error)(&error)
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

#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
pub(crate) async fn emit_stronghold_status_change(status: &crate::StrongholdStatus) {
    let listeners = stronghold_status_change_listeners().lock().await;
    for listener in listeners.deref() {
        (listener.on_event)(&status)
    }
}

/// Listen to stronghold status change events.
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
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
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
pub async fn remove_stronghold_status_change_listener(id: &EventId) {
    remove_event_listener(id, stronghold_status_change_listeners()).await;
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

    #[tokio::test]
    async fn balance_events() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        let account = account_handle.read().await;
        let account_id = account.id().to_string();
        on_balance_change(move |event| {
            assert!(event.account_id == &account_id);
            assert!(event.balance_change.spent == 5);
            assert!(event.balance_change.received == 0);
        })
        .await;

        emit_balance_change(
            &account,
            &crate::test_utils::generate_random_iota_address(),
            BalanceChange::spent(5),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn on_new_transaction_event() {
        let account_id = "new-tx";
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let message_ = message.clone();

        on_new_transaction(move |event| {
            assert!(event.account_id == account_id);
            assert!(event.message == message_);
        })
        .await;

        emit_transaction_event(TransactionEventType::NewTransaction, account_id, &message).await;
    }

    #[tokio::test]
    async fn on_reattachment_event() {
        let account_id = "reattachment";
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let message_ = message.clone();

        on_reattachment(move |event| {
            assert!(event.account_id == account_id);
            assert!(event.message == message_);
        })
        .await;

        emit_transaction_event(TransactionEventType::Reattachment, account_id, &message).await;
    }

    #[tokio::test]
    async fn on_broadcast_event() {
        let account_id = "broadcast";
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let message_ = message.clone();

        on_broadcast(move |event| {
            assert!(event.account_id == account_id);
            assert!(event.message == message_);
        })
        .await;

        emit_transaction_event(TransactionEventType::Broadcast, account_id, &message).await;
    }

    #[tokio::test]
    async fn on_confirmation_state_change_event() {
        let account_id = "confirm";
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let message_ = message.clone();
        let confirmed = true;

        on_confirmation_state_change(move |event| {
            assert!(event.account_id == account_id);
            assert!(event.message == message_);
            assert!(event.confirmed == confirmed);
        })
        .await;

        emit_confirmation_state_change(account_id, &message, confirmed).await;
    }
}

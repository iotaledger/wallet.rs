// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountIdentifier, address::Address, message::Message};

use getset::Getters;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

/// The balance change event data.
#[derive(Getters, Serialize)]
#[getset(get = "pub")]
pub struct BalanceEvent<'a> {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    account_id: &'a AccountIdentifier,
    /// The associated address.
    address: &'a Address,
    /// The new balance.
    balance: u64,
}

impl<'a> BalanceEvent<'a> {
    #[doc(hidden)]
    pub fn cloned_address(&self) -> Address {
        self.address.clone()
    }
}

/// A transaction-related event data.
#[derive(Getters, Serialize)]
#[getset(get = "pub")]
pub struct TransactionEvent<'a> {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    account_id: &'a AccountIdentifier,
    /// The event message.
    message: &'a Message,
}

impl<'a> TransactionEvent<'a> {
    #[doc(hidden)]
    pub fn cloned_message(&self) -> Message {
        self.message.clone()
    }
}

/// A transaction-related event data.
#[derive(Getters, Serialize)]
#[getset(get = "pub")]
pub struct TransactionConfirmationChangeEvent<'a> {
    #[serde(rename = "accountId")]
    /// The associated account identifier.
    account_id: &'a AccountIdentifier,
    /// The event message.
    message: &'a Message,
    /// The confirmed state of the transaction.
    confirmed: bool,
}

impl<'a> TransactionConfirmationChangeEvent<'a> {
    #[doc(hidden)]
    pub fn cloned_message(&self) -> Message {
        self.message.clone()
    }
}

struct BalanceEventHandler {
    /// The on event callback.
    on_event: Box<dyn Fn(&BalanceEvent<'_>) + Send>,
}

struct ErrorHandler {
    /// The on error callback.
    on_error: Box<dyn Fn(&crate::Error) + Send>,
}

#[derive(PartialEq)]
pub(crate) enum TransactionEventType {
    NewTransaction,
    Reattachment,
    Broadcast,
}

struct TransactionEventHandler {
    event_type: TransactionEventType,
    /// The on event callback.
    on_event: Box<dyn Fn(&TransactionEvent<'_>) + Send>,
}

struct TransactionConfirmationChangeEventHandler {
    /// The on event callback.
    on_event: Box<dyn Fn(&TransactionConfirmationChangeEvent<'_>) + Send>,
}

type BalanceListeners = Arc<Mutex<Vec<BalanceEventHandler>>>;
type TransactionListeners = Arc<Mutex<Vec<TransactionEventHandler>>>;
type TransactionConfirmationChangeListeners = Arc<Mutex<Vec<TransactionConfirmationChangeEventHandler>>>;
type ErrorListeners = Arc<Mutex<Vec<ErrorHandler>>>;

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

/// Listen to balance changes.
pub fn on_balance_change<F: Fn(&BalanceEvent<'_>) + Send + 'static>(cb: F) {
    let mut l = balance_listeners()
        .lock()
        .expect("Failed to lock balance_listeners: on_balance_change()");
    l.push(BalanceEventHandler { on_event: Box::new(cb) })
}

/// Emits a balance change event.
pub(crate) fn emit_balance_change(account_id: &AccountIdentifier, address: &Address, balance: u64) {
    let listeners = balance_listeners()
        .lock()
        .expect("Failed to lock balance_listeners: emit_balance_change()");
    let event = BalanceEvent {
        account_id,
        address: &address,
        balance,
    };
    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

/// Emits a transaction-related event.
pub(crate) fn emit_transaction_event(
    event_type: TransactionEventType,
    account_id: &AccountIdentifier,
    message: &Message,
) {
    let listeners = transaction_listeners()
        .lock()
        .expect("Failed to lock balance_listeners: emit_balance_change()");
    let event = TransactionEvent {
        account_id,
        message: &message,
    };
    for listener in listeners.deref() {
        if listener.event_type == event_type {
            (listener.on_event)(&event);
        }
    }
}

/// Emits a transaction confirmation state change event.
pub(crate) fn emit_confirmation_state_change(account_id: &AccountIdentifier, message: &Message, confirmed: bool) {
    let listeners = transaction_confirmation_change_listeners()
        .lock()
        .expect("Failed to lock transaction_confirmation_change_listeners: emit_confirmation_state_change()");
    let event = TransactionConfirmationChangeEvent {
        account_id,
        message: &message,
        confirmed,
    };
    for listener in listeners.deref() {
        (listener.on_event)(&event);
    }
}

/// Adds a transaction-related event listener.
fn add_transaction_listener<F: Fn(&TransactionEvent<'_>) + Send + 'static>(event_type: TransactionEventType, cb: F) {
    let mut l = transaction_listeners()
        .lock()
        .expect("Failed to lock transaction_listeners: add_transaction_listener()");
    l.push(TransactionEventHandler {
        event_type,
        on_event: Box::new(cb),
    })
}

/// Listen to new messages.
pub fn on_new_transaction<F: Fn(&TransactionEvent<'_>) + Send + 'static>(cb: F) {
    add_transaction_listener(TransactionEventType::NewTransaction, cb);
}

/// Listen to transaction confirmation state change.
pub fn on_confirmation_state_change<F: Fn(&TransactionConfirmationChangeEvent<'_>) + Send + 'static>(cb: F) {
    let mut l = transaction_confirmation_change_listeners()
        .lock()
        .expect("Failed to lock transaction_confirmation_change_listeners: on_confirmation_state_change()");
    l.push(TransactionConfirmationChangeEventHandler { on_event: Box::new(cb) })
}

/// Listen to transaction reattachment.
pub fn on_reattachment<F: Fn(&TransactionEvent<'_>) + Send + 'static>(cb: F) {
    add_transaction_listener(TransactionEventType::Reattachment, cb);
}

/// Listen to transaction broadcast.
pub fn on_broadcast<F: Fn(&TransactionEvent<'_>) + Send + 'static>(cb: F) {
    add_transaction_listener(TransactionEventType::Broadcast, cb);
}

pub(crate) fn emit_error(error: &crate::Error) {
    let listeners = error_listeners()
        .lock()
        .expect("Failed to lock error_listeners: emit_error()");
    for listener in listeners.deref() {
        (listener.on_error)(&error)
    }
}

/// Listen to errors.
pub fn on_error<F: Fn(&crate::Error) + Send + 'static>(cb: F) {
    let mut l = error_listeners()
        .lock()
        .expect("Failed to lock error_listeners: on_error()");
    l.push(ErrorHandler { on_error: Box::new(cb) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::AccountIdentifier,
        address::{AddressBuilder, IotaAddress},
    };
    use iota::message::prelude::Ed25519Address;
    use rusty_fork::rusty_fork_test;

    fn _create_and_drop_error() {
        let _ = crate::Error::AccountNotFound;
    }

    // have to fork this test so other errors dropped doesn't affect it
    rusty_fork_test! {
        #[test]
        fn error_events() {
            on_error(|error| {
                assert!(matches!(error, crate::Error::AccountNotFound));
            });
            _create_and_drop_error();
        }
    }

    #[test]
    fn balance_events() {
        on_balance_change(|event| {
            assert!(event.account_id == &AccountIdentifier::Id(hex::encode([1; 32])));
            assert!(event.balance == 0);
        });

        emit_balance_change(
            &AccountIdentifier::Id(hex::encode([1; 32])),
            &AddressBuilder::new()
                .address(IotaAddress::Ed25519(Ed25519Address::new([0; 32])))
                .balance(0)
                .key_index(0)
                .outputs(vec![])
                .build()
                .expect("failed to build address"),
            0,
        );
    }

    #[test]
    fn on_new_transaction_event() {
        let account_id: AccountIdentifier = "new-tx".to_string().into();
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let account_id_ = account_id.clone();
        let message_ = message.clone();

        on_new_transaction(move |event| {
            assert!(event.account_id == &account_id_);
            assert!(event.message == &message_);
        });

        emit_transaction_event(TransactionEventType::NewTransaction, &account_id, &message);
    }

    #[test]
    fn on_reattachment_event() {
        let account_id: AccountIdentifier = "reattachment".to_string().into();
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let account_id_ = account_id.clone();
        let message_ = message.clone();

        on_reattachment(move |event| {
            assert!(event.account_id == &account_id_);
            assert!(event.message == &message_);
        });

        emit_transaction_event(TransactionEventType::Reattachment, &account_id, &message);
    }

    #[test]
    fn on_broadcast_event() {
        let account_id: AccountIdentifier = "broadcast".to_string().into();
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let account_id_ = account_id.clone();
        let message_ = message.clone();

        on_broadcast(move |event| {
            assert!(event.account_id == &account_id_);
            assert!(event.message == &message_);
        });

        emit_transaction_event(TransactionEventType::Broadcast, &account_id, &message);
    }

    #[test]
    fn on_confirmation_state_change_event() {
        let account_id: AccountIdentifier = "confirm".to_string().into();
        let message = crate::test_utils::GenerateMessageBuilder::default().build();
        let account_id_ = account_id.clone();
        let message_ = message.clone();
        let confirmed = true;

        on_confirmation_state_change(move |event| {
            assert!(event.account_id == &account_id_);
            assert!(event.message == &message_);
            assert!(event.confirmed == confirmed);
        });

        emit_confirmation_state_change(&account_id, &message, confirmed);
    }
}

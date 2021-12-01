// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod types;

use types::{Event, WalletEvent, WalletEventType};

use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result},
};

type Handler<T> = Box<dyn Fn(&T) + Send + Sync + 'static>;

pub struct EventEmitter {
    handlers: HashMap<WalletEventType, Vec<Handler<Event>>>,
}

impl EventEmitter {
    /// Creates a new instance of `EventEmitter`.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers function `handler` as a listener for a `WalletEventType`. There may be
    /// multiple listeners for a single event.
    pub fn on<F>(&mut self, events: Vec<WalletEventType>, handler: F)
    where
        F: Fn(&Event) + 'static + Clone + Send + Sync,
    {
        // if no event is provided the handler is registered for all event types
        if events.is_empty() {
            // we could use a crate like strum or a macro to iterate over all values, but not sure if it's worth it
            for event_type in vec![
                WalletEventType::BalanceChange,
                WalletEventType::TransactionInclusion,
                WalletEventType::TransferProgress,
                WalletEventType::ConsolidationRequired,
                #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
                WalletEventType::LedgerAddressGeneration,
            ] {
                let event_handlers = self.handlers.entry(event_type).or_insert_with(Vec::new);
                event_handlers.push(Box::new(handler.clone()));
            }
        }
        for event in events.into_iter() {
            let event_handlers = self.handlers.entry(event).or_insert_with(Vec::new);
            event_handlers.push(Box::new(handler.clone()));
        }
    }

    /// Invokes all listeners of `event`, passing a reference to `payload` as an
    /// argument to each of them.
    pub fn emit(&self, account_index: usize, event: WalletEvent) {
        let event_type = match &event {
            WalletEvent::BalanceChange(_) => WalletEventType::BalanceChange,
            WalletEvent::TransactionInclusion(_) => WalletEventType::TransactionInclusion,
            WalletEvent::TransferProgress(_) => WalletEventType::TransferProgress,
            WalletEvent::ConsolidationRequired => WalletEventType::ConsolidationRequired,
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            WalletEvent::LedgerAddressGeneration(_) => WalletEventType::LedgerAddressGeneration,
        };
        let event = Event { account_index, event };
        if let Some(handlers) = self.handlers.get(&event_type) {
            for handler in handlers {
                handler(&event);
            }
        }
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for EventEmitter {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "event_types_with_handlers: {:?}",
            self.handlers.keys().collect::<Vec<&WalletEventType>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{
        types::{TransactionInclusionEvent, TransferProgressEvent, WalletEvent, WalletEventType},
        EventEmitter,
    };
    use crate::account::types::InclusionState;

    use iota_client::bee_message::payload::transaction::TransactionId;

    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn events() {
        let mut emitter = EventEmitter::new();
        let event_counter = Arc::new(AtomicUsize::new(0));

        // single event
        emitter.on(vec![WalletEventType::ConsolidationRequired], |name| {
            // println!("ConsolidationRequired: {:?}", name);
        });

        // listen to two events
        emitter.on(
            vec![
                WalletEventType::TransferProgress,
                WalletEventType::ConsolidationRequired,
            ],
            move |name| {
                // println!("TransferProgress or ConsolidationRequired: {:?}", name);
            },
        );

        // listen to all events
        let event_counter_clone = Arc::clone(&event_counter);
        emitter.on(vec![], move |name| {
            // println!("Any event: {:?}", name);
            event_counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // emit events
        emitter.emit(0, WalletEvent::ConsolidationRequired);
        emitter.emit(0, WalletEvent::TransferProgress(TransferProgressEvent::SyncingAccount));
        emitter.emit(
            0,
            WalletEvent::TransactionInclusion(TransactionInclusionEvent {
                transaction_id: TransactionId::from_str(
                    "2289d9981fb23cc5f4f6c2742685eeb480f8476089888aa886a18232bad81989",
                )
                .expect("Invalid tx id"),
                inclusion_state: InclusionState::Confirmed,
            }),
        );

        assert_eq!(3, event_counter.load(Ordering::SeqCst));
        for _ in 0..1_000_000 {
            emitter.emit(0, WalletEvent::ConsolidationRequired);
        }
        assert_eq!(1_000_003, event_counter.load(Ordering::SeqCst));
    }
}

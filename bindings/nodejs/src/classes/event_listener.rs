// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use neon::prelude::*;
use std::{
    convert::TryInto,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
};

pub use iota_wallet::{
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    actor::{Message as WalletMessage, MessageType, Response, ResponseType, WalletMessageHandler},
    address::parse as parse_address,
    event::{
        on_balance_change, on_broadcast, on_confirmation_state_change, on_error, on_new_transaction, on_reattachment,
        on_stronghold_status_change, on_transfer_progress, remove_balance_change_listener, remove_broadcast_listener,
        remove_confirmation_state_change_listener, remove_error_listener, remove_new_transaction_listener,
        remove_reattachment_listener, remove_stronghold_status_change_listener, remove_transfer_progress_listener,
        EventId,
    },
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
    Error,
};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub enum EventType {
    ErrorThrown,
    BalanceChange,
    NewTransaction,
    ConfirmationStateChange,
    Reattachment,
    Broadcast,
    StrongholdStatusChange,
    TransferProgress,
}

impl TryFrom<&str> for EventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let event_type = match value {
            "ErrorThrown" => EventType::ErrorThrown,
            "BalanceChange" => EventType::BalanceChange,
            "NewTransaction" => EventType::NewTransaction,
            "ConfirmationStateChange" => EventType::ConfirmationStateChange,
            "Reattachment" => EventType::Reattachment,
            "Broadcast" => EventType::Broadcast,
            "StrongholdStatusChange" => EventType::StrongholdStatusChange,
            "TransferProgress" => EventType::TransferProgress,
            _ => return Err(format!("invalid event name {}", value)),
        };
        Ok(event_type)
    }
}

type JsCallback = Root<JsFunction<JsObject>>;
type JsCallbackMap = HashMap<EventType, Vec<(JsCallback, EventId)>>;
pub(crate) struct EventListener {
    channel: Channel,
    callbacks: Arc<Mutex<JsCallbackMap>>,
}

impl Finalize for EventListener {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        for (event_type, mut callbacks) in self.callbacks.lock().unwrap().drain() {
            for (cb, event_id) in callbacks.drain(..) {
                crate::RUNTIME.spawn(async move {
                    EventListener::remove_event_listeners(event_type, &event_id).await;
                });
                cb.drop(cx);
            }
        }
    }
}

impl EventListener {
    fn new(channel: Channel) -> Arc<Self> {
        Arc::new(Self {
            channel,
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn call(&self, message: String, event_type: EventType) {
        let callbacks = self.callbacks.clone();
        self.channel.send(move |mut cx| {
            if let Some(cbs) = callbacks.lock().unwrap().get(&event_type) {
                for (cb, _) in cbs {
                    let cb = cb.to_inner(&mut cx);
                    let this = cx.undefined();
                    let args = vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.string(message.clone()).upcast::<JsValue>(),
                    ];

                    cb.call(&mut cx, this, args)?;
                }
            };

            Ok(())
        });
    }

    async fn remove_event_listeners(event_type: EventType, event_id: &[u8; 32]) {
        match event_type {
            EventType::ErrorThrown => remove_error_listener(event_id),
            EventType::BalanceChange => remove_balance_change_listener(event_id).await,
            EventType::NewTransaction => remove_new_transaction_listener(event_id).await,
            EventType::ConfirmationStateChange => remove_confirmation_state_change_listener(event_id).await,
            EventType::Reattachment => remove_reattachment_listener(event_id).await,
            EventType::Broadcast => remove_broadcast_listener(event_id).await,
            EventType::StrongholdStatusChange => remove_stronghold_status_change_listener(event_id).await,
            EventType::TransferProgress => remove_transfer_progress_listener(event_id).await,
        };
    }

    pub async fn add_event_listener<F: Fn(String, EventType) + Send + 'static>(
        event_type: EventType,
        callback: F,
    ) -> [u8; 32] {
        match event_type {
            EventType::ErrorThrown => on_error(move |error| {
                let _ = callback(serde_json::to_string(&error).unwrap(), event_type);
            }),
            EventType::BalanceChange => {
                on_balance_change(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::NewTransaction => {
                on_new_transaction(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::ConfirmationStateChange => {
                on_confirmation_state_change(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::Reattachment => {
                on_reattachment(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::Broadcast => {
                on_broadcast(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::StrongholdStatusChange => {
                on_stronghold_status_change(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
            EventType::TransferProgress => {
                on_transfer_progress(move |event| {
                    let _ = callback(serde_json::to_string(&event).unwrap(), event_type);
                })
                .await
            }
        }
    }
}

pub(crate) fn remove_event_listeners(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let event_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let event_type: EventType = event_name.as_str().try_into().expect("unknown event name");
    let event_handler = Arc::clone(&&cx.argument::<JsBox<Arc<EventListener>>>(1)?);
    let mut cb_storage = event_handler.callbacks.lock().unwrap();
    let cbs = cb_storage.remove(&event_type);
    let cbs_amount = cb_storage.len();

    if let Some(cbs) = cbs {
        for (cb, event_id) in cbs {
            crate::RUNTIME.spawn(async move {
                EventListener::remove_event_listeners(event_type, &event_id).await;
            });
            cb.drop(&mut cx);
        }
    }

    Ok(cx.number(cbs_amount as f64))
}

pub(crate) fn listen(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let event_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let event_type: EventType = event_name.as_str().try_into().expect("unknown event name");
    let event_handler = Arc::clone(&&cx.argument::<JsBox<Arc<EventListener>>>(1)?);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    crate::RUNTIME.spawn(async move {
        let cloned_eh = event_handler.clone();
        let event_id = EventListener::add_event_listener(event_type, move |message: String, event_type: EventType| {
            cloned_eh.call(message, event_type);
        })
        .await;

        match event_handler.callbacks.lock().unwrap().entry(event_type) {
            Entry::Vacant(e) => {
                e.insert(vec![(callback, event_id)]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push((callback, event_id));
            }
        }
    });

    Ok(cx.undefined())
}

pub(crate) fn event_listener_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<EventListener>>> {
    let channel = cx.channel();
    let event_handler = EventListener::new(channel);

    Ok(cx.boxed(event_handler))
}

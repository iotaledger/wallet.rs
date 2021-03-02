// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    convert::TryFrom,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use iota_wallet::event::{
    on_balance_change, on_broadcast, on_confirmation_state_change, on_error, on_new_transaction, on_reattachment,
    on_transfer_progress, EventId,
};
use neon::prelude::*;

pub enum EventType {
    ErrorThrown,
    BalanceChange,
    NewTransaction,
    ConfirmationStateChange,
    Reattachment,
    Broadcast,
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
            "TransferProgress" => EventType::TransferProgress,
            _ => return Err(format!("invalid event name {}", value)),
        };
        Ok(event_type)
    }
}

async fn listen(event_type: EventType, sender: Sender<String>) -> EventId {
    match event_type {
        EventType::ErrorThrown => on_error(move |error| {
            let _ = sender.send(serde_json::to_string(&error).unwrap());
        }),
        EventType::BalanceChange => {
            on_balance_change(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
        EventType::NewTransaction => {
            on_new_transaction(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
        EventType::ConfirmationStateChange => {
            on_confirmation_state_change(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
        EventType::Reattachment => {
            on_reattachment(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
        EventType::Broadcast => {
            on_broadcast(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
        EventType::TransferProgress => {
            on_transfer_progress(move |event| {
                let _ = sender.send(serde_json::to_string(&event).unwrap());
            })
            .await
        }
    }
}

struct WaitForEventTask(Arc<Mutex<Receiver<String>>>);

impl Task for WaitForEventTask {
    type Output = String;
    type Error = String;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let rx = self
            .0
            .lock()
            .map_err(|_| "Could not obtain lock on receiver".to_string())?;
        rx.recv().map_err(|e| e.to_string())
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match result {
            Ok(s) => Ok(cx.string(s)),
            Err(e) => cx.throw_error(format!("ReceiveTask error: {}", e)),
        }
    }
}

pub struct EventListener {
    rx: Arc<Mutex<Receiver<String>>>,
}

declare_types! {
    pub class JsEventListener for EventListener {
        init(mut cx) {
            let event = EventType::try_from(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid event type");
            let (tx, rx) = channel();

            crate::block_on(listen(event, tx));

            Ok(EventListener {
                rx: Arc::new(Mutex::new(rx)),
            })
        }

        method poll(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();

            let rx = cx.borrow(&this, |listener| Arc::clone(&listener.rx));
            let receive_task = WaitForEventTask(rx);

            receive_task.schedule(cb);

            Ok(JsUndefined::new().upcast())
        }
    }
}

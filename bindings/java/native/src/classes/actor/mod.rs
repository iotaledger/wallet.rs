// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod wallet_actor;

mod actors;
use actors::{dispatch, DispatchMessage, WalletActor, WalletActorMsg};

use bee_common::logger::logger_init;
pub use bee_common::logger::LoggerConfigBuilder;
use iota_wallet::{
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    client::drop_all as drop_clients,
    event::{
        on_balance_change, on_broadcast, on_confirmation_state_change, on_error, on_migration_progress,
        on_new_transaction, on_reattachment, on_stronghold_status_change, on_transfer_progress,
        remove_balance_change_listener, remove_broadcast_listener, remove_confirmation_state_change_listener,
        remove_error_listener, remove_migration_progress_listener, remove_new_transaction_listener,
        remove_reattachment_listener, remove_stronghold_status_change_listener, remove_transfer_progress_listener,
        EventId,
    },
};
use once_cell::sync::Lazy;
use riker::actors::*;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::event_manager::EventType;

pub const POLLING_INTERVAL_MS: u64 = 30_000;

struct WalletActorData {
    listeners: Vec<(EventId, EventType)>,
    actor: ActorRef<WalletActorMsg>,
}

type WalletActors = Arc<AsyncMutex<HashMap<String, WalletActorData>>>;
type MessageReceiver = Box<dyn Fn(String) + Send + 'static>;
type MessageReceivers = Arc<Mutex<HashMap<String, MessageReceiver>>>;

fn wallet_actors() -> &'static WalletActors {
    static ACTORS: Lazy<WalletActors> = Lazy::new(Default::default);
    &ACTORS
}

fn message_receivers() -> &'static MessageReceivers {
    static RECEIVERS: Lazy<MessageReceivers> = Lazy::new(Default::default);
    &RECEIVERS
}

pub async fn init<A: Into<String>, F: Fn(String) + Send + 'static>(
    actor_id: A,
    message_receiver: F,
    storage_path: Option<impl AsRef<Path>>,
) {
    let actor_id = actor_id.into();

    let mut actors = wallet_actors().lock().await;

    let manager = AccountManager::builder()
        .with_storage(
            match storage_path {
                Some(path) => path.as_ref().to_path_buf(),
                None => PathBuf::from(DEFAULT_STORAGE_FOLDER),
            },
            None,
        )
        .unwrap() // safe to unwrap, the storage password is None ^
        .with_polling_interval(Duration::from_millis(POLLING_INTERVAL_MS))
        .with_sync_spent_outputs()
        .finish()
        .await
        .expect("failed to init account manager");

    iota_wallet::with_actor_system(|sys| {
        let wallet_actor = sys.actor_of_args::<WalletActor, _>(&actor_id, manager).unwrap();

        actors.insert(
            actor_id.to_string(),
            WalletActorData {
                listeners: Vec::new(),
                actor: wallet_actor,
            },
        );

        let mut message_receivers = message_receivers()
            .lock()
            .expect("Failed to lock message_receivers: init()");
        message_receivers.insert(actor_id, Box::new(message_receiver));
    })
    .await;
}

pub async fn remove_event_listeners<A: Into<String>>(actor_id: A) {
    let mut actors = wallet_actors().lock().await;
    if let Some(actor_data) = actors.get_mut(&actor_id.into()) {
        remove_event_listeners_internal(&actor_data.listeners).await;
        actor_data.listeners = Vec::new();
    }
}

async fn remove_event_listeners_internal(listeners: &[(EventId, EventType)]) {
    for (event_id, event_type) in listeners.iter() {
        match event_type {
            &EventType::ErrorThrown => remove_error_listener(event_id),
            &EventType::BalanceChange => remove_balance_change_listener(event_id).await,
            &EventType::NewTransaction => remove_new_transaction_listener(event_id).await,
            &EventType::ConfirmationStateChange => remove_confirmation_state_change_listener(event_id).await,
            &EventType::Reattachment => remove_reattachment_listener(event_id).await,
            &EventType::Broadcast => remove_broadcast_listener(event_id).await,
            &EventType::StrongholdStatusChange => remove_stronghold_status_change_listener(event_id).await,
            &EventType::TransferProgress => remove_transfer_progress_listener(event_id).await,
            &EventType::MigrationProgress => remove_migration_progress_listener(event_id).await,
        };
    }
}

pub async fn destroy<A: Into<String>>(actor_id: A) {
    let mut actors = wallet_actors().lock().await;
    let actor_id = actor_id.into();

    if let Some(actor_data) = actors.remove(&actor_id) {
        remove_event_listeners_internal(&actor_data.listeners).await;

        actor_data.actor.tell(actors::KillMessage, None);
        iota_wallet::with_actor_system(|sys| {
            sys.stop(&actor_data.actor);
        })
        .await;

        // delay to wait for the actor to be killed
        tokio::time::sleep(Duration::from_millis(500)).await;
        drop_clients().await;

        let mut message_receivers = message_receivers()
            .lock()
            .expect("Failed to lock message_receivers: respond()");
        message_receivers.remove(&actor_id);
    }
}

pub fn init_logger(config: LoggerConfigBuilder) {
    logger_init(config.finish()).expect("failed to init logger");
}

#[derive(Deserialize)]
pub(crate) struct MessageFallback {
    #[serde(rename = "actorId")]
    pub(crate) actor_id: String,
    pub(crate) id: Option<String>,
}

pub async fn send_message(serialized_message: String) {
    let data = match serde_json::from_str::<DispatchMessage>(&serialized_message) {
        Ok(message) => {
            let actors = wallet_actors().lock().await;

            let actor_id = message.actor_id.to_string();
            if let Some(actor) = actors.get(&actor_id) {
                match dispatch(&actor.actor, message).await {
                    Ok(response) => Some((response, actor_id)),
                    Err(e) => Some((Some(e), actor_id)),
                }
            } else {
                Some((
                    Some(format!(
                        r#"{{ "type": "ActorNotInitialised", "payload": "{}" }}"#,
                        message.actor_id
                    )),
                    message.actor_id,
                ))
            }
        }
        Err(error) => {
            if let Ok(message) = serde_json::from_str::<MessageFallback>(&serialized_message) {
                Some((
                    Some(format!(
                        r#"{{
                            "type": "Error",
                            "id": {},
                            "payload": {{ 
                                "type": "InvalidMessage",
                                "message": {},
                                "error": {}
                             }}
                        }}"#,
                        match message.id {
                            Some(id) => serde_json::Value::String(id),
                            None => serde_json::Value::Null,
                        },
                        serialized_message,
                        serde_json::Value::String(error.to_string()),
                    )),
                    message.actor_id,
                ))
            } else {
                log::error!("[FIREFLY] backend sendMessage error: {:?}", error);
                None
            }
        }
    };

    if let Some((message, actor_id)) = data {
        if let Some(message) = message {
            respond(&actor_id, message).expect("actor dropped");
        } else {
            log::error!(
                "[FIREFLY] unexpected empty response for message `{}`, the channel was dropped",
                serialized_message
            );
        }
    }
}

#[derive(Serialize)]
struct EventResponse<T: Serialize> {
    id: String,
    #[serde(rename = "type")]
    _type: EventType,
    payload: T,
}

impl<T: Serialize> EventResponse<T> {
    fn new<S: Into<String>>(id: S, event: EventType, payload: T) -> Self {
        Self {
            id: id.into(),
            _type: event,
            payload,
        }
    }
}

fn serialize_event<T: Serialize, S: Into<String>>(id: S, event: EventType, payload: T) -> String {
    serde_json::to_string(&EventResponse::new(id, event, payload)).unwrap()
}

fn respond<A: AsRef<str>>(actor_id: A, message: String) -> Result<(), String> {
    let message_receivers = message_receivers()
        .lock()
        .expect("Failed to lock message_receivers: respond()");

    if let Some(callback) = message_receivers.get(actor_id.as_ref()) {
        callback(message);
        Ok(())
    } else {
        Err("message receiver dropped".to_string())
    }
}

pub async fn listen<A: Into<String>, S: Into<String>>(actor_id: A, id: S, event_type: EventType) {
    let id = id.into();
    let actor_id = actor_id.into();

    let actor_id_ = actor_id.clone();
    let event_type_ = event_type.clone();

    let event_id = match event_type {
        EventType::ErrorThrown => on_error(move |error| {
            let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &error));
        }),
        EventType::BalanceChange => {
            on_balance_change(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::NewTransaction => {
            on_new_transaction(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::ConfirmationStateChange => {
            on_confirmation_state_change(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::Reattachment => {
            on_reattachment(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::Broadcast => {
            on_broadcast(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::StrongholdStatusChange => {
            on_stronghold_status_change(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::TransferProgress => {
            on_transfer_progress(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
        EventType::MigrationProgress => {
            on_migration_progress(move |event| {
                let _ = respond(&actor_id, serialize_event(id.clone(), event_type, &event));
            })
            .await
        }
    };

    let mut actors = wallet_actors().lock().await;
    let actor = actors.get_mut(&actor_id_).expect("actor not initialised");
    actor.listeners.push((event_id, event_type_));
}

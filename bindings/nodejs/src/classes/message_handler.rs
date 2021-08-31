// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use neon::prelude::*;
use std::sync::Arc;

use tokio::sync::mpsc::unbounded_channel;

use serde::Deserialize;

use std::time::Duration;

pub use iota_wallet::{
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    actor::{AccountIdentifier, Message as WalletMessage, MessageType, Response, ResponseType, WalletMessageHandler},
    address::parse as parse_address,
    event::{
        on_balance_change, on_broadcast, on_confirmation_state_change, on_error, on_migration_progress,
        on_new_transaction, on_reattachment, on_stronghold_status_change, on_transfer_progress,
        remove_balance_change_listener, remove_broadcast_listener, remove_confirmation_state_change_listener,
        remove_error_listener, remove_migration_progress_listener, remove_new_transaction_listener,
        remove_reattachment_listener, remove_stronghold_status_change_listener, remove_transfer_progress_listener,
        EventId,
    },
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
    Error,
};

// The dummy id is used for the actor system, which we don't currently use.
const DUMMY_ID: &str = "1";

#[derive(Deserialize, Clone)]
pub(crate) struct DispatchMessage {
    #[serde(flatten)]
    pub(crate) message: MessageType,
}

pub struct MessageHandler {
    channel: Channel,
    message_handler: WalletMessageHandler,
}

impl Finalize for MessageHandler {}
impl MessageHandler {
    fn new(channel: Channel, options: String) -> Arc<Self> {
        let options = match serde_json::from_str::<crate::types::ManagerOptions>(&options) {
            Ok(options) => options,
            Err(e) => {
                log::debug!("{:?}", e);
                crate::types::ManagerOptions::default()
            }
        };

        let mut manager = AccountManager::builder()
            .with_storage(&options.storage_path, options.storage_password.as_deref())
            .expect("failed to init storage");
        if !options.automatic_output_consolidation {
            manager = manager.with_automatic_output_consolidation_disabled();
        }
        if options.sync_spent_outputs {
            manager = manager.with_sync_spent_outputs();
        }
        if options.persist_events {
            manager = manager.with_event_persistence();
        }
        if options.allow_create_multiple_empty_accounts {
            manager = manager.with_multiple_empty_accounts();
        }
        if options.skip_polling {
            manager = manager.with_skip_polling();
        }
        if let Some(polling_interval) = options.polling_interval {
            manager = manager.with_polling_interval(Duration::from_secs(polling_interval));
        }
        if let Some(threshold) = options.output_consolidation_threshold {
            manager = manager.with_output_consolidation_threshold(threshold);
        }
        let manager = crate::RUNTIME
            .block_on(manager.finish())
            .expect("error initializing account manager");
        let message_handler = WalletMessageHandler::with_manager(manager);

        Arc::new(Self {
            channel,
            message_handler,
        })
    }

    async fn send_message(&self, serialized_message: String) -> (String, bool) {
        log::debug!("{}", serialized_message);
        match serde_json::from_str::<DispatchMessage>(&serialized_message) {
            Ok(message) => {
                let (response_tx, mut response_rx) = unbounded_channel();
                let wallet_message = WalletMessage::new(DUMMY_ID, message.message.clone(), response_tx);

                self.message_handler.handle(wallet_message).await;
                let response = response_rx.recv().await;
                if let Some(res) = response {
                    let mut is_err = matches!(res.response(), ResponseType::Error(_) | ResponseType::Panic(_));

                    let msg = match serde_json::to_string(&res) {
                        Ok(msg) => msg,
                        Err(e) => {
                            is_err = true;
                            serde_json::to_string(&Response::new(
                                DUMMY_ID,
                                message.message,
                                ResponseType::Error(e.into()),
                            ))
                            .expect("The response is generated manually, so unwrap is safe.")
                        }
                    };

                    (msg, is_err)
                } else {
                    ("No response".to_string(), true)
                }
            }
            Err(e) => {
                log::debug!("{:?}", e);
                (format!("Couldn't parse to message with error - {:?}", e), true)
            }
        }
    }
}

pub fn message_handler_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<MessageHandler>>> {
    let options = cx.argument::<JsString>(0)?;
    let options = options.value(&mut cx);
    let channel = cx.channel();
    let message_handler = MessageHandler::new(channel, options);

    Ok(cx.boxed(message_handler))
}

pub fn send_message(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let message = cx.argument::<JsString>(0)?;
    let message = message.value(&mut cx);
    let message_handler = Arc::clone(&&cx.argument::<JsBox<Arc<MessageHandler>>>(1)?);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    crate::RUNTIME.spawn(async move {
        let (response, is_error) = message_handler.send_message(message).await;
        log::debug!("{:?}", response);
        message_handler.channel.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                if is_error {
                    cx.string(response.clone()).upcast::<JsValue>()
                } else {
                    cx.undefined().upcast::<JsValue>()
                },
                cx.string(response).upcast::<JsValue>(),
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

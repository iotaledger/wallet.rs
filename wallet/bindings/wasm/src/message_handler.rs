// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use iota_wallet::{
    events::types::{Event, WalletEventType},
    message_interface::{
        create_message_handler, init_logger as init_logger_rust, ManagerOptions, Message, Response,
        WalletMessageHandler,
    },
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

/// The Wallet message handler.
#[wasm_bindgen(js_name = MessageHandler)]
pub struct MessageHandler {
    handler: Rc<RefCell<Option<WalletMessageHandler>>>,
}

/// Creates a message handler with the given options.
#[wasm_bindgen(js_name = messageHandlerNew)]
#[allow(non_snake_case)]
pub fn message_handler_new(options: String) -> Result<MessageHandler, JsValue> {
    let manager_options = match serde_json::from_str::<ManagerOptions>(&options) {
        Ok(options) => Some(options),
        Err(e) => return Err(e.to_string().into()),
    };

    let wallet_message_handler = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async move { create_message_handler(manager_options).await })
        .expect("error initializing account manager");

    Ok(MessageHandler {
        handler: Rc::new(RefCell::new(Some(wallet_message_handler))),
    })
}

#[wasm_bindgen]
pub async fn destroy(message_handler: &MessageHandler) -> Result<(), JsValue> {
    *message_handler.handler.borrow_mut() = None;
    Ok(())
}

pub async fn init_logger(config: String) -> Result<(), JsValue> {
    init_logger_rust(config).map_err(|e| e.to_string())?;
    Ok(())
}

/// Handles a message, returns the response as a JSON-encoded string.
///
/// Returns an error if the response itself is an error or panic.
#[wasm_bindgen(js_name = sendMessageAsync)]
#[allow(non_snake_case, clippy::await_holding_refcell_ref)]
pub async fn send_message_async(message: String, message_handler: &MessageHandler) -> Result<String, JsValue> {
    let message_handler = message_handler.handler.borrow_mut();
    let message: Message = serde_json::from_str(&message).map_err(|err| err.to_string())?;

    let response = message_handler.as_ref().unwrap().send_message(message).await;
    match response {
        Response::Error(e) => Err(e.to_string().into()),
        Response::Panic(p) => Err(p.into()),
        _ => Ok(serde_json::to_string(&response)
            .map_err(|err| JsValue::from_str(&format!("Client MessageHandler failed to serialize response: {err}")))?),
    }
}

/// It takes a list of event types, registers a callback function, and then listens for events of those
/// types
///
/// Arguments:
///
/// * `vec`: An array of strings that represent the event types you want to listen to.
/// * `callback`: A JavaScript function that will be called when a wallet event occurs.
/// * `message_handler`: This is the same message handler that we used in the previous section.
#[wasm_bindgen]
pub async fn listen(
    vec: js_sys::Array,
    callback: js_sys::Function,
    message_handler: &MessageHandler,
) -> Result<JsValue, JsValue> {
    let mut event_types = vec![];
    for event_type in vec.keys() {
        // We know the built-in iterator for set elements won't throw
        // exceptions, so just unwrap the element.
        let event_type = event_type.unwrap().as_string().unwrap();
        let wallet_event_type = WalletEventType::try_from(event_type.as_str()).map_err(JsValue::from)?;
        event_types.push(wallet_event_type);
    }

    let (tx, mut rx): (UnboundedSender<Event>, UnboundedReceiver<Event>) = unbounded_channel();
    message_handler
        .handler
        .borrow()
        .as_ref()
        .expect("message handler not initialised")
        .listen(event_types, move |wallet_event| {
            tx.send(wallet_event.clone()).unwrap();
        })
        .await;

    // Spawn on the same thread a continuous loop to check the channel
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(wallet_event) = rx.recv().await {
            callback
                .call1(
                    &JsValue::NULL,
                    &JsValue::from(serde_json::to_string(&wallet_event).unwrap()),
                )
                // Safe to unwrap, our callback has no return
                .unwrap();
        }
        // No more links to the unbounded_channel, exit loop
    });

    Ok(JsValue::UNDEFINED)
}

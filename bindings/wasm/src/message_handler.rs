// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::rc::Rc;

use iota_wallet::{
    events::types::{Event, WalletEventType},
    message_interface::{
        create_message_handler, init_logger as init_logger_rust, ManagerOptions, Message, Response,
        WalletMessageHandler,
    },
};

use tokio::sync::mpsc::unbounded_channel;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::future_to_promise;

/// The Client message handler.
#[wasm_bindgen(js_name = MessageHandler)]
pub struct MessageHandler {
    handler: Rc<WalletMessageHandler>,
}

/// Creates a message handler with the given client options.
#[wasm_bindgen(js_name = messageHandlerNew)]
#[allow(non_snake_case)]
pub fn message_handler_new(options: String) -> Result<MessageHandler, JsValue> {
    let manager_options = match serde_json::from_str::<ManagerOptions>(&options) {
        Ok(options) => Some(options),
        Err(e) => {return Err(e.to_string().into())},
    };
    
    let wallet_message_handler = tokio::runtime::Builder::new_current_thread().build().unwrap()
        .block_on(async move { create_message_handler(manager_options).await })
        .expect("error initializing account manager");

    Ok(MessageHandler {
        handler: Rc::new(wallet_message_handler),
    })
}

#[wasm_bindgen]
pub async fn destroy(message_handler: &MessageHandler) -> Result<(), JsValue> {
    //Rc::into_raw(message_handler.handler);
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
#[allow(non_snake_case)]
pub async fn send_message_async(message: String, message_handler: &MessageHandler) -> Result<String, JsValue> {
    let message_handler: Rc<WalletMessageHandler> = Rc::clone(&message_handler.handler);
    let message: Message = serde_json::from_str(&message).map_err(|err| err.to_string())?;

    let response = message_handler.send_message(message).await;
    match response {
        Response::Error(e) => Err(e.to_string().into()),
        Response::Panic(p) => Err(p.into()),
        _ => Ok(serde_json::to_string(&response).map_err(|err| {
            JsValue::from_str(&format!("Client MessageHandler failed to serialize response: {err}"))
        })?),
    }
}

use std::sync::{Arc, Mutex};

/// 
#[wasm_bindgen]
pub async fn listen(message_handler: &MessageHandler, vec: js_sys::Array, callback: &js_sys::Function) -> Result<JsValue, JsValue> {
    let mut event_types = vec![];
    for i in 0..vec.length() {
        let event_type = vec.get(i).as_string().unwrap();
        let wallet_event_type =
            WalletEventType::try_from(event_type.as_str()).or_else(|e| { return Err(e)})?;
        event_types.push(wallet_event_type);
    }

    message_handler
        .handler
        .listen(event_types, move |event_data| {
            callback.call1(&JsValue::NULL, &JsValue::from( serde_json::to_string(&event_data).unwrap()));
        })
        .await;

    Ok(JsValue::UNDEFINED)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Promise<string>")]
    pub type PromiseString;
}
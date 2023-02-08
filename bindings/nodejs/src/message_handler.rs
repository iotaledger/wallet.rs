// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use iota_wallet::{
    events::types::{Event, WalletEventType},
    message_interface::{
        create_message_handler, init_logger as init_logger_rust, ManagerOptions, Message, Response,
        WalletMessageHandler,
    },
};
use neon::prelude::*;
use tokio::sync::RwLock;

// Wrapper so we can destroy the MessageHandler
pub type MessageHandlerWrapperInner = Arc<RwLock<Option<MessageHandler>>>;
// Wrapper because we can't impl Finalize on MessageHandlerWrapperInner
pub struct MessageHandlerWrapper(pub MessageHandlerWrapperInner);
impl Finalize for MessageHandlerWrapper {}

pub struct MessageHandler {
    channel: Channel,
    wallet_message_handler: WalletMessageHandler,
}

type JsCallback = Root<JsFunction<JsObject>>;

impl MessageHandler {
    fn new(channel: Channel, options: String) -> Self {
        let manager_options = match serde_json::from_str::<ManagerOptions>(&options) {
            Ok(options) => Some(options),
            Err(e) => {
                log::debug!("{:?}", e);
                None
            }
        };

        let wallet_message_handler = crate::RUNTIME
            .block_on(async move { create_message_handler(manager_options).await })
            .expect("error initializing account manager");

        Self {
            channel,
            wallet_message_handler,
        }
    }

    async fn send_message(&self, serialized_message: String) -> (String, bool) {
        match serde_json::from_str::<Message>(&serialized_message) {
            Ok(message) => {
                let res = self.wallet_message_handler.send_message(message).await;
                let mut is_err = matches!(res, Response::Error(_) | Response::Panic(_));

                let msg = match serde_json::to_string(&res) {
                    Ok(msg) => msg,
                    Err(e) => {
                        is_err = true;
                        serde_json::to_string(&Response::Error(e.into()))
                            .expect("the response is generated manually, so unwrap is safe.")
                    }
                };

                (msg, is_err)
            }
            Err(e) => {
                log::debug!("{:?}", e);
                (
                    serde_json::to_string(&Response::Error(e.into()))
                        .expect("the response is generated manually, so unwrap is safe."),
                    true,
                )
            }
        }
    }
}

impl Finalize for MessageHandler {}

fn call_event_callback(channel: &neon::event::Channel, event_data: Event, callback: Arc<JsCallback>) {
    channel.send(move |mut cx| {
        let cb = (*callback).to_inner(&mut cx);
        let this = cx.undefined();
        let args = vec![
            cx.undefined().upcast::<JsValue>(),
            cx.string(serde_json::to_string(&event_data).unwrap())
                .upcast::<JsValue>(),
        ];

        cb.call(&mut cx, this, args)?;

        Ok(())
    });
}

pub fn init_logger(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsString>(0)?.value(&mut cx);
    init_logger_rust(config).expect("failed to init logger");
    Ok(cx.undefined())
}

pub fn message_handler_new(mut cx: FunctionContext) -> JsResult<JsBox<MessageHandlerWrapper>> {
    let options = cx.argument::<JsString>(0)?;
    let options = options.value(&mut cx);
    let channel = cx.channel();
    let message_handler = MessageHandler::new(channel, options);

    Ok(cx.boxed(MessageHandlerWrapper(Arc::new(RwLock::new(Some(message_handler))))))
}

pub fn send_message(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let message = cx.argument::<JsString>(0)?;
    let message = message.value(&mut cx);
    let message_handler = Arc::clone(&&cx.argument::<JsBox<MessageHandlerWrapper>>(1)?.0);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    crate::RUNTIME.spawn(async move {
        if let Some(message_handler) = &*message_handler.read().await {
            let (response, is_error) = message_handler.send_message(message).await;
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
        } else {
            panic!("Message handler got destroyed")
        }
    });

    Ok(cx.undefined())
}

pub fn listen(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let js_arr_handle: Handle<JsArray> = cx.argument(0)?;
    let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;
    let mut event_types = vec![];
    for event_string in vec {
        let event_type = event_string.downcast_or_throw::<JsString, FunctionContext>(&mut cx)?;
        let wallet_event_type =
            WalletEventType::try_from(event_type.value(&mut cx).as_str()).or_else(|e| cx.throw_error(e))?;
        event_types.push(wallet_event_type);
    }

    let callback = Arc::new(cx.argument::<JsFunction>(1)?.root(&mut cx));
    let message_handler = Arc::clone(&&cx.argument::<JsBox<MessageHandlerWrapper>>(2)?.0);

    crate::RUNTIME.spawn(async move {
        if let Some(message_handler) = &*message_handler.read().await {
            let channel = message_handler.channel.clone();
            message_handler
                .wallet_message_handler
                .listen(event_types, move |event_data| {
                    call_event_callback(&channel, event_data.clone(), callback.clone())
                })
                .await;
        } else {
            panic!("Message handler got destroyed")
        }
    });

    Ok(cx.undefined())
}

pub fn destroy(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let message_handler = Arc::clone(&&cx.argument::<JsBox<MessageHandlerWrapper>>(0)?.0);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    crate::RUNTIME.spawn(async move {
        *message_handler.write().await = None;
        deferred.settle_with(&channel, move |mut cx| Ok(cx.undefined()));
    });
    Ok(promise)
}

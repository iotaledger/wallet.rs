// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::needless_borrow)]

pub mod types;
// pub mod event_listener;
pub mod message_handler;
// pub use event_listener::*;
pub use message_handler::*;

use bee_common::logger::{logger_init, LoggerConfigBuilder};
use neon::prelude::*;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

pub fn init_logger(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsString>(0)?.value(&mut cx);
    let config: LoggerConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    logger_init(config.finish()).expect("failed to init logger");
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Message handler methods.
    cx.export_function("sendMessage", message_handler::send_message)?;
    cx.export_function("messageHandlerNew", message_handler::message_handler_new)?;

    // cx.export_function("eventListenerNew", event_listener::event_listener_new)?;
    // cx.export_function("listen", event_listener::listen)?;
    // cx.export_function("removeEventListeners", event_listener::remove_event_listeners)?;

    cx.export_function("initLogger", init_logger)?;
    Ok(())
}

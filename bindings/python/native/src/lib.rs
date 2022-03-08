// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod types;

use iota_client::common::logger::{logger_init, LoggerConfigBuilder};
use iota_wallet::{
    events::types::WalletEventType,
    message_interface::{ManagerOptions, MessageType},
};
use types::*;

use once_cell::sync::OnceCell;
use pyo3::{prelude::*, wrap_pyfunction};
use std::sync::Mutex;
use tokio::runtime::Runtime;

/// Use one runtime.
pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

#[pyfunction]
/// Init the logger of wallet library.
pub fn init_logger(config: String) -> PyResult<()> {
    let config: LoggerConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    logger_init(config.finish()).expect("failed to init logger");
    Ok(())
}

#[pyfunction]
/// Create message handler for python-side usage.
pub fn create_message_handler(options: String) -> Result<WalletMessageHandler> {
    let options = match serde_json::from_str::<ManagerOptions>(&options) {
        Ok(options) => Some(options),
        Err(e) => {
            log::debug!("Error options input {:?}", e);
            None
        }
    };
    let message_handler =
        crate::block_on(async { iota_wallet::message_interface::create_message_handler(options).await })?;

    Ok(WalletMessageHandler {
        wallet_message_handler: message_handler,
    })
}

#[pyfunction]
/// Send message through handler.
pub fn send_message(handle: &WalletMessageHandler, message_type: String) -> Result<String> {
    let message_type = match serde_json::from_str::<MessageType>(&message_type) {
        Ok(message_type) => message_type,
        Err(e) => {
            panic!("Cannot create message handler! {:?}", e);
        }
    };
    let response = crate::block_on(async {
        iota_wallet::message_interface::send_message(&handle.wallet_message_handler, message_type).await
    });
    Ok(serde_json::to_string(&response)?)
}

#[pyfunction]
/// Listen to events.
pub fn listen(handle: &WalletMessageHandler, events: Vec<String>, handler: PyObject) {
    let mut rust_events = Vec::new();
    for event in events {
        let event = match serde_json::from_str::<WalletEventType>(&event) {
            Ok(event) => event,
            Err(e) => {
                panic!("Error event to listen! {:?}", e);
            }
        };
        rust_events.push(event);
    }
    crate::block_on(async {
        iota_wallet::message_interface::listen(&handle.wallet_message_handler, rust_events, move |_| {
            let gil = Python::acquire_gil();
            let py = gil.python();
            handler.call0(py).unwrap();
        })
        .await;
    });
}

/// IOTA Wallet implemented in Rust for Python binding.
#[pymodule]
fn iota_wallet(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_logger, m)?).unwrap();
    m.add_function(wrap_pyfunction!(create_message_handler, m)?).unwrap();
    m.add_function(wrap_pyfunction!(send_message, m)?).unwrap();
    m.add_function(wrap_pyfunction!(listen, m)?).unwrap();
    Ok(())
}

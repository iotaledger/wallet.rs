// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod classes;
pub mod types;

use classes::event::*;
use once_cell::sync::OnceCell;
use pyo3::{prelude::*, wrap_pyfunction};
use tokio::runtime::Runtime;
use types::*;

use std::sync::Mutex;

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

/// IOTA Wallet implemented in Rust and binded by Python.
#[pymodule]
fn iota_wallet(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AccountInitialiser>()?;
    m.add_class::<AccountHandle>()?;
    m.add_class::<SyncedAccount>()?;
    m.add_class::<AccountSynchronizer>()?;
    m.add_class::<Transfer>()?;
    m.add_class::<TransferWithOutputs>()?;
    m.add_class::<AccountManager>()?;
    m.add_function(wrap_pyfunction!(on_balance_change, m)?).unwrap();
    m.add_function(wrap_pyfunction!(remove_balance_change_listener, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(on_new_transaction, m)?).unwrap();
    m.add_function(wrap_pyfunction!(remove_new_transaction_listener, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(on_confirmation_state_change, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(remove_confirmation_state_change_listener, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(on_reattachment, m)?).unwrap();
    m.add_function(wrap_pyfunction!(remove_reattachment_listener, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(on_broadcast, m)?).unwrap();
    m.add_function(wrap_pyfunction!(remove_broadcast_listener, m)?).unwrap();
    m.add_function(wrap_pyfunction!(on_error, m)?).unwrap();
    m.add_function(wrap_pyfunction!(remove_error_listener, m)?).unwrap();
    m.add_function(wrap_pyfunction!(on_stronghold_status_change, m)?)
        .unwrap();
    m.add_function(wrap_pyfunction!(remove_stronghold_status_change_listener, m)?)
        .unwrap();
    Ok(())
}

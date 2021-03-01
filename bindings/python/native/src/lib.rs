// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod classes;
pub mod types;

use once_cell::sync::OnceCell;
use pyo3::prelude::*;
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
    m.add_class::<AccountManager>()?;
    Ok(())
}

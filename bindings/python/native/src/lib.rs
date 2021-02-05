// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod classes;
pub mod types;

use pyo3::prelude::*;
use types::*;

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

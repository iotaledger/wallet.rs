// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Used in verifying correct binding
pub mod verifylink;

mod classes;
mod types;

#[cfg(target_os = "android")]
mod android_c_headers;
#[cfg(target_os = "android")]
mod java_glue;


#[cfg(target_os = "android")]
pub use crate::java_glue::*;

pub use crate::{classes::*, types::*};

pub use anyhow::{Error, Result};

use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tokio::runtime::Runtime;

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

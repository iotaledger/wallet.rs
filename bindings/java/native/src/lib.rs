// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod jni_c_header;

// Used in verifying correct binding
pub mod verifylink;

mod classes;
mod java_glue;
mod types;

pub use crate::{classes::*, java_glue::*, types::*};

pub use smol::block_on;

pub use anyhow::{Error, Result};

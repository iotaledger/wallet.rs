// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

/// The wallet-tester error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Client error.
    #[error("client error: {0}")]
    Client(#[from] iota_wallet::iota_client::Error),
    /// I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Json error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Wallet error.
    #[error("wallet error: {0}")]
    Wallet(#[from] iota_wallet::Error),
}

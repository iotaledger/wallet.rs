// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

/// The wallet-tester error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Check error.
    #[error("{0}")]
    Check(String),
    /// Client error.
    #[error("client error: {0}")]
    Client(#[from] iota_wallet::iota_client::Error),
    /// Invalid field.
    #[error("invalid field: {0}")]
    InvalidField(&'static str),
    /// I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Json error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Logger error.
    #[error("logger error: {0}")]
    Logger(#[from] fern_logger::Error),
    /// Missing field.
    #[error("missing field: {0}")]
    MissingField(&'static str),
    // /// Types error.
    // #[error("types error: {0}")]
    // Types(#[from] iota_wallet::iota_client::types::Error),
    /// Unexpected error.
    #[error("unexpected error: expected {expected:?}, got {actual:?}")]
    Unexpected { expected: String, actual: String },
    /// Wallet error.
    #[error("wallet error: {0}")]
    Wallet(#[from] iota_wallet::Error),
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]
#![allow(unused_variables, dead_code)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The actor interface for the library.
pub mod actor;
/// The address module.
pub mod address;
/// The client module.
pub mod client;
/// The event module.
pub mod event;
/// The message module.
pub mod message;
/// The monitor module.
pub mod monitor;
pub(crate) mod serde;
/// Signing interfaces.
pub mod signing;
/// The storage module.
pub mod storage;
#[cfg(feature = "stronghold")]
pub(crate) mod stronghold;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, WalletError>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tokio::runtime::Runtime;

/// The wallet error type.
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    /// Unknown error.
    #[error("`{0}`")]
    UnknownError(String),
    /// Generic error.
    #[error("{0}")]
    GenericError(#[from] anyhow::Error),
    /// IO error.
    #[error("`{0}`")]
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
    JsonError(#[from] serde_json::error::Error),
    /// stronghold client error.
    #[cfg(feature = "stronghold")]
    #[error("`{0}`")]
    StrongholdError(#[from] stronghold::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(#[from] iota::client::Error),
    /// rusqlite error.
    #[cfg(feature = "sqlite")]
    #[error("`{0}`")]
    SqliteError(#[from] rusqlite::Error),
    /// url parse error.
    #[error("`{0}`")]
    UrlError(#[from] url::ParseError),
    /// Unexpected node response error.
    #[error("`{0}`")]
    UnexpectedResponse(String),
    /// Message above max depth error (message timestamp above 10 minutes).
    #[error("message is above the max depth")]
    MessageAboveMaxDepth,
    /// Message is already confirmed.
    #[error("message is already confirmed")]
    MessageAlreadyConfirmed,
    /// Message not found.
    #[error("message not found")]
    MessageNotFound,
    /// Node list is empty.
    #[error("empty node list")]
    EmptyNodeList,
    /// Address length invalid.
    #[error("unexpected address length")]
    InvalidAddressLength,
    /// Transaction id length response invalid.
    #[error("unexpected transaction_id length")]
    InvalidTransactionIdLength,
    /// Message id length response invalid.
    #[error("unexpected message_id length")]
    InvalidMessageIdLength,
    /// bech32 error.
    #[error("`{0}`")]
    Bech32Error(#[from] bech32::Error),
    /// An account is already imported.
    #[error("acount `{alias}` already imported")]
    AccountAlreadyImported {
        /// the account alias.
        alias: String,
    },
    /// Storage file doesn't exist
    #[error("storage file doesn't exist")]
    StorageDoesntExist,
    /// Insufficient funds to send transfer.
    #[error("insufficient funds")]
    InsufficientFunds,
    /// Message isn't empty (has history or balance).
    #[error("message has history or balance")]
    MessageNotEmpty,
    /// Latest account is empty (doesn't have history and balance) - can't create account.
    #[error("can't create accounts when the latest account doesn't have message history and balance")]
    LatestAccountIsEmpty,
    /// Transfer amount can't be zero.
    #[error("transfer amount can't be zero")]
    ZeroAmount,
    /// Account not found
    #[error("account not found")]
    AccountNotFound,
    /// invalid remainder value target address defined on `RemainderValueStrategy`.
    /// the address must belong to the account.
    #[error("the remainder value address doesn't belong to the account")]
    InvalidRemainderValueAddress,
    /// Stronghold instance not initialised with its password.
    #[error("stronghold not initialised")]
    StrongholdNotInitialised,
}

impl Drop for WalletError {
    fn drop(&mut self) {
        event::emit_error(self);
    }
}
pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

#[cfg(test)]
mod test_utils {
    use super::account_manager::AccountManager;
    use once_cell::sync::OnceCell;
    use rand::{thread_rng, Rng};
    use std::path::PathBuf;

    static MANAGER_INSTANCE: OnceCell<AccountManager> = OnceCell::new();
    pub fn get_account_manager() -> &'static AccountManager {
        MANAGER_INSTANCE.get_or_init(|| {
            let storage_path: String = thread_rng().gen_ascii_chars().take(10).collect();
            let storage_path = PathBuf::from(format!("./example-database/{}", storage_path));

            let mut manager = AccountManager::with_storage_path(storage_path).unwrap();
            crate::block_on(manager.set_stronghold_password("password")).unwrap();
            manager
        })
    }
}

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

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, WalletError>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use stronghold::Stronghold;
use tokio::runtime::Runtime;

static STRONGHOLD_INSTANCE: OnceCell<Arc<Mutex<HashMap<PathBuf, Stronghold>>>> = OnceCell::new();
static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();

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
    /// stronghold error.
    #[error("`{0}`")]
    StrongholdError(#[from] stronghold::VaultError),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(#[from] iota::client::Error),
    /// rusqlite error.
    #[cfg(any(feature = "sqlite", feature = "stronghold"))]
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
}

impl Drop for WalletError {
    fn drop(&mut self) {
        event::emit_error(self);
    }
}

pub(crate) fn init_stronghold(stronghold_path: &PathBuf, stronghold: Stronghold) {
    let mut stronghold_map = STRONGHOLD_INSTANCE.get_or_init(Default::default).lock().unwrap();
    stronghold_map.insert(stronghold_path.to_path_buf(), stronghold);
}

pub(crate) fn remove_stronghold(stronghold_path: PathBuf) {
    let mut stronghold_map = STRONGHOLD_INSTANCE.get_or_init(Default::default).lock().unwrap();
    stronghold_map.remove(&stronghold_path);
}

pub(crate) fn with_stronghold_from_path<T, F: FnOnce(&Stronghold) -> crate::Result<T>>(
    path: &PathBuf,
    cb: F,
) -> crate::Result<T> {
    let stronghold_map = STRONGHOLD_INSTANCE.get_or_init(Default::default).lock().unwrap();
    if let Some(stronghold) = stronghold_map.get(path) {
        cb(stronghold)
    } else {
        Err(anyhow::anyhow!("should initialize stronghold instance before using it").into())
    }
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

pub(crate) fn enter<R, C: FnOnce() -> R>(cb: C) -> R {
    let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().enter(cb)
}

#[cfg(test)]
mod test_utils {
    use super::account_manager::AccountManager;
    use iota::pow::providers::{Provider as PowProvider, ProviderBuilder as PowProviderBuilder};
    use once_cell::sync::OnceCell;
    use rand::{thread_rng, Rng};
    use std::{path::PathBuf, sync::Mutex, time::Duration};

    static MANAGER_INSTANCE: OnceCell<Mutex<AccountManager>> = OnceCell::new();
    pub fn get_account_manager() -> &'static Mutex<AccountManager> {
        MANAGER_INSTANCE.get_or_init(|| {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                let storage_path: String = thread_rng().gen_ascii_chars().take(10).collect();
                let storage_path = PathBuf::from(format!("./example-database/{}", storage_path));

                let mut manager = AccountManager::builder()
                    .with_storage_path(storage_path)
                    .with_polling_interval(Duration::from_secs(4))
                    .finish()
                    .await
                    .unwrap();
                manager.set_stronghold_password("password").await.unwrap();
                Mutex::new(manager)
            })
        })
    }

    /// The miner builder.
    #[derive(Default)]
    pub struct NoopNonceProviderBuilder;

    impl PowProviderBuilder for NoopNonceProviderBuilder {
        type Provider = NoopNonceProvider;

        fn new() -> Self {
            Self::default()
        }

        fn finish(self) -> NoopNonceProvider {
            NoopNonceProvider {}
        }
    }

    /// The miner used for PoW
    pub struct NoopNonceProvider;

    impl PowProvider for NoopNonceProvider {
        type Builder = NoopNonceProviderBuilder;
        type Error = crate::WalletError;

        fn nonce(&self, bytes: &[u8], target_score: f64) -> std::result::Result<u64, Self::Error> {
            Ok(0)
        }
    }
}

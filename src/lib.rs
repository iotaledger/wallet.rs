//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]
#![allow(unused_variables, dead_code)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
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
/// The storage module.
pub mod storage;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, WalletError>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use serde::ser::{SerializeStruct, Serializer};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stronghold::Stronghold;

static STRONGHOLD_INSTANCE: OnceCell<Arc<Mutex<HashMap<PathBuf, Stronghold>>>> = OnceCell::new();

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
    #[error(
        "can't create accounts when the latest account doesn't have message history and balance"
    )]
    LatestAccountIsEmpty,
    /// Transfer amount can't be zero.
    #[error("transfer amount can't be zero")]
    ZeroAmount,
}

impl serde::Serialize for WalletError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn serialize_variant<S: Serializer>(
            serializer: S,
            variant_name: &str,
            message: Option<&str>,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_struct("WalletError", 2)?;
            state.serialize_field("type", variant_name)?;
            state.serialize_field("error", &message)?;
            state.end()
        }
        match self {
            Self::UnknownError(error) => serialize_variant(serializer, "UnknownError", Some(error)),
            Self::GenericError(error) => {
                serialize_variant(serializer, "GenericError", Some(&error.to_string()))
            }
            Self::IoError(error) => {
                serialize_variant(serializer, "IoError", Some(&error.to_string()))
            }
            Self::JsonError(error) => {
                serialize_variant(serializer, "JsonError", Some(&error.to_string()))
            }
            Self::StrongholdError(error) => {
                serialize_variant(serializer, "StrongholdError", Some(&error.to_string()))
            }
            Self::ClientError(error) => {
                serialize_variant(serializer, "ClientError", Some(&error.to_string()))
            }
            Self::SqliteError(error) => {
                serialize_variant(serializer, "SqliteError", Some(&error.to_string()))
            }
            Self::UrlError(error) => {
                serialize_variant(serializer, "UrlError", Some(&error.to_string()))
            }
            Self::UnexpectedResponse(error) => {
                serialize_variant(serializer, "UnexpectedResponse", Some(&error))
            }
            Self::MessageAboveMaxDepth => {
                serialize_variant(serializer, "MessageAboveMaxDepth", None)
            }
            Self::MessageAlreadyConfirmed => {
                serialize_variant(serializer, "MessageAlreadyConfirmed", None)
            }
            Self::MessageNotFound => serialize_variant(serializer, "MessageNotFound", None),
            Self::EmptyNodeList => serialize_variant(serializer, "EmptyNodeList", None),
            Self::InvalidAddressLength => {
                serialize_variant(serializer, "InvalidAddressLength", None)
            }
            Self::InvalidTransactionIdLength => serializer.serialize_newtype_variant(
                "WalletError",
                14,
                "InvalidTransactionIdLength",
                "",
            ),
            Self::InvalidMessageIdLength => {
                serialize_variant(serializer, "InvalidMessageIdLength", None)
            }
            Self::Bech32Error(error) => {
                serialize_variant(serializer, "Bech32Error", Some(&error.to_string()))
            }
            Self::AccountAlreadyImported { alias } => serialize_variant(
                serializer,
                "AccountAlreadyImported",
                Some(&format!("account {} already imported", alias)),
            ),
            Self::StorageDoesntExist => serialize_variant(serializer, "StorageDoesntExist", None),
            Self::InsufficientFunds => serialize_variant(serializer, "InsufficientFunds", None),
            Self::MessageNotEmpty => serialize_variant(serializer, "MessageNotEmpty", None),
            Self::LatestAccountIsEmpty => {
                serialize_variant(serializer, "LatestAccountIsEmpty", None)
            }
            Self::ZeroAmount => serialize_variant(serializer, "ZeroAmount", None),
        }
    }
}

impl Drop for WalletError {
    fn drop(&mut self) {
        event::emit_error(self);
    }
}

pub(crate) fn init_stronghold(stronghold_path: &PathBuf, stronghold: Stronghold) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.insert(stronghold_path.to_path_buf(), stronghold);
}

pub(crate) fn remove_stronghold(stronghold_path: PathBuf) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.remove(&stronghold_path);
}

pub(crate) fn with_stronghold_from_path<T, F: FnOnce(&Stronghold) -> T>(
    path: &PathBuf,
    cb: F,
) -> T {
    let stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    if let Some(stronghold) = stronghold_map.get(path) {
        cb(stronghold)
    } else {
        panic!("should initialize stronghold instance before using it")
    }
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

            let manager = AccountManager::with_storage_path(storage_path).unwrap();
            manager.set_stronghold_password("password").unwrap();
            manager
        })
    }
}

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
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
pub(crate) mod stronghold;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();

/// The wallet error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error.
    #[error("`{0}`")]
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
    JsonError(#[from] serde_json::error::Error),
    /// stronghold client error.
    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
    #[error("`{0}`")]
    StrongholdError(#[from] stronghold::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(#[from] iota::client::Error),
    /// rusqlite error.
    #[cfg(feature = "sqlite-storage")]
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
    /// Storage access error.
    #[error("error accessing storage: {0}")]
    Storage(String),
    /// Panic error.
    #[error("a panic happened: {0}")]
    Panic(String),
    /// Error on `internal_transfer` when the destination account address list is empty
    #[error("destination account has no addresses")]
    TransferDestinationEmpty,
    /// Invalid message identifier.
    #[error("invalid message id received by node")]
    InvalidMessageId,
    /// Invalid transaction identifier.
    #[error("invalid transaction id received by node")]
    InvalidTransactionId,
    /// Address build error: required field not filled.
    #[error("address build error, field `{0}` is required")]
    AddressBuildRequiredField(AddressBuildRequiredField),
    /// Account initialisation error: required field not filled.
    #[error("account initialisation error, field `{0}` is required")]
    AccountInitialiseRequiredField(AccountInitialiseRequiredField),
    /// Error that happens when the stronghold snapshot wasn't loaded.
    /// The snapshot is loaded through the
    /// [AccountManager#set_stronghold_password](struct.AccountManager.html#method.set_stronghold_password).
    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
    #[error("stronghold not loaded")]
    StrongholdNotLoaded,
    /// Invalid hex string.
    #[error("invalid hex string received: {0}")]
    Hex(#[from] hex::FromHexError),
    /// Error from bee_message crate.
    #[error("{0}")]
    BeeMessage(iota::message::Error),
    /// Transaction output amount can't be zero.
    #[error("transaction output amount can't be zero")]
    OutputAmountIsZero,
    /// invalid BIP32 derivation path.
    #[error("invalid BIP32 derivation path: {0}")]
    InvalidDerivationPath(String),
    /// Failed to generate private key from BIP32 path.
    #[error("failed to generate private key from derivation path")]
    FailedToGeneratePrivateKey(bee_signing_ext::binary::BIP32Path),
    /// Failed to parse date string.
    #[error("error parsing date: {0}")]
    ParseDate(#[from] chrono::ParseError),
    /// Error from iota crypto.rs
    #[error("crypto error: {0}")]
    Crypto(crypto::Error),
    /// Path provided to `import_accounts` isn't a file
    #[error("provided backup path isn't a file")]
    BackupNotFile,
    /// Backup `destination` argument is invalid
    #[error("backup destination must be a directory and it must exist")]
    InvalidBackupDestination,
    /// the storage adapter isn't set
    #[error("the storage adapter isn't set; use the AccountManagerBuilder's `with_storage` method or one of the default storages with the crate features `sqlite-storage` and `stronghold-storage`.")]
    StorageAdapterNotDefined,
}

impl From<iota::message::Error> for Error {
    fn from(error: iota::message::Error) -> Self {
        Self::BeeMessage(error)
    }
}

impl From<crypto::Error> for Error {
    fn from(error: crypto::Error) -> Self {
        Self::Crypto(error)
    }
}

/// Each of the account initialisation required fields.
#[derive(Debug)]
pub enum AccountInitialiseRequiredField {
    /// `signer_type` field.
    SignerType,
}

impl std::fmt::Display for AccountInitialiseRequiredField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Each of the address builder required fields.
#[derive(Debug)]
pub enum AddressBuildRequiredField {
    /// address field.
    Address,
    /// balance field.
    Balance,
    /// key_index field.
    KeyIndex,
    /// outputs field.
    Outputs,
}

impl std::fmt::Display for AddressBuildRequiredField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl Drop for Error {
    fn drop(&mut self) {
        event::emit_error(self);
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
    use rand::{thread_rng, Rng};
    use std::{path::PathBuf, thread::sleep, time::Duration};

    static POLLING_INTERVAL: Duration = Duration::from_secs(2);

    pub async fn get_account_manager() -> AccountManager {
        std::fs::create_dir_all("./example-database").unwrap();
        let storage_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let storage_path = PathBuf::from(format!("./example-database/{}", storage_path));

        let mut manager = AccountManager::builder()
            .with_storage_path(storage_path)
            .with_polling_interval(POLLING_INTERVAL)
            .finish()
            .await
            .unwrap();
        manager.set_stronghold_password("password").await.unwrap();
        manager
    }

    pub fn wait_accounts_save() {
        sleep(POLLING_INTERVAL * 2);
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
        type Error = crate::Error;

        fn nonce(&self, bytes: &[u8], target_score: f64) -> std::result::Result<u64, Self::Error> {
            Ok(0)
        }
    }
}

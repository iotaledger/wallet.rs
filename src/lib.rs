// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]

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

pub use stronghold::set_password_clear_interval as set_stronghold_password_clear_interval;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::{path::PathBuf, sync::Mutex};
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
    /// invalid BIP32 derivation path.
    #[error("invalid BIP32 derivation path: {0}")]
    InvalidDerivationPath(String),
    /// Failed to parse date string.
    #[error("error parsing date: {0}")]
    ParseDate(#[from] chrono::ParseError),
    /// Error from iota crypto.rs
    #[error("crypto error: {0}")]
    Crypto(crypto::Error),
    /// Path provided to `import_accounts` isn't a valid file
    #[error("provided backup path isn't a valid file")]
    InvalidBackupFile,
    /// Backup `destination` argument is invalid
    #[error("backup destination must be a directory and it must exist")]
    InvalidBackupDestination,
    /// the storage adapter isn't set
    #[error("the storage adapter isn't set; use the AccountManagerBuilder's `with_storage` method or one of the default storages with the crate features `sqlite-storage` and `stronghold-storage`.")]
    StorageAdapterNotDefined,
    /// Mnemonic generation error.
    #[error("mnemonic encode error")]
    MnemonicEncode,
    /// Invalid mnemonic error
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    /// Can't import accounts because the storage already exist
    #[error("failed to restore backup: storage file already exists")]
    StorageExists,
    /// Storage adapter not defined for the given storage path.
    #[error(
        "storage adapter not set for path `{0}`; please use the method `with_storage` on the AccountManager builder"
    )]
    StorageAdapterNotSet(PathBuf),
    /// error decrypting stored account using provided encryptionKey
    #[error("failed to decrypt account")]
    AccountDecrypt,
    /// Ledger transport error
    #[error("ledger transport error")]
    LedgerMiscError,
    /// Dongle Locked
    #[error("ledger locked")]
    LedgerDongleLocked,
    /// Denied by User
    #[error("denied by user")]
    LedgerDeniedByUser,
    /// Ledger Device not found
    #[error("ledger device not found")]
    LedgerDeviceNotFound,
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

pub(crate) fn spawn<F>(future: F)
where
    F: futures::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().spawn(future);
}

/// Access the stronghold's actor system.
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
pub async fn with_actor_system<F: FnOnce(&riker::actors::ActorSystem)>(cb: F) {
    let runtime = self::stronghold::actor_runtime().lock().await;
    cb(&runtime.stronghold.system)
}

#[cfg(test)]
mod test_utils {
    use super::{account_manager::AccountManager, signing::SignerType};
    use iota::pow::providers::{Provider as PowProvider, ProviderBuilder as PowProviderBuilder};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::{path::PathBuf, time::Duration};

    static POLLING_INTERVAL: Duration = Duration::from_secs(2);

    struct TestSigner {}

    #[async_trait::async_trait]
    impl crate::signing::Signer for TestSigner {
        async fn store_mnemonic(&self, _: &PathBuf, _mnemonic: String) -> crate::Result<()> {
            Ok(())
        }

        async fn generate_address(
            &self,
            _account: &crate::account::Account,
            _address_index: usize,
            _internal: bool,
            _metadata: crate::signing::GenerateAddressMetadata,
        ) -> crate::Result<iota::Address> {
            let mut address = [0; iota::ED25519_ADDRESS_LENGTH];
            crypto::rand::fill(&mut address).unwrap();
            Ok(iota::Address::Ed25519(iota::Ed25519Address::new(address)))
        }

        async fn sign_message<'a>(
            &self,
            _account: &crate::account::Account,
            _essence: &iota::TransactionPayloadEssence,
            _inputs: &mut Vec<crate::signing::TransactionInput>,
            _metadata: crate::signing::SignMessageMetadata<'a>,
        ) -> crate::Result<Vec<iota::UnlockBlock>> {
            Ok(Vec::new())
        }
    }

    pub fn signer_type() -> SignerType {
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        let signer_type = SignerType::Stronghold;
        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        let signer_type = SignerType::Custom("".to_string());
        signer_type
    }

    pub async fn get_account_manager() -> AccountManager {
        let storage_path = loop {
            let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
            let storage_path = PathBuf::from(format!("./test-storage/{}", storage_path));
            if !storage_path.exists() {
                break storage_path;
            }
        };

        let mut manager = AccountManager::builder()
            .with_storage_path(storage_path)
            .with_polling_interval(POLLING_INTERVAL)
            .finish()
            .await
            .unwrap();

        manager.set_storage_password("password").await.unwrap();

        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        crate::signing::set_signer(signer_type(), TestSigner {}).await;
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        manager.set_stronghold_password("password").await.unwrap();

        manager.store_mnemonic(signer_type(), None).await.unwrap();

        manager
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

        fn nonce(&self, _bytes: &[u8], _target_score: f64) -> std::result::Result<u64, Self::Error> {
            Ok(0)
        }
    }
}

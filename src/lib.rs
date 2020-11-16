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
pub(crate) mod serde;
/// The storage module.
pub mod storage;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, WalletError>;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
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
    /// Account not found
    #[error("account not found")]
    AccountNotFound,
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

pub(crate) fn is_stronghold_initialised(path: &PathBuf) -> bool {
    let stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.contains_key(path)
}

pub(crate) fn with_stronghold_from_path<T, F: FnOnce(&Stronghold) -> T>(
    path: &PathBuf,
    cb: F,
) -> T {
    let stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    println!("{:?}", stronghold_map.keys());
    println!("{:?}", path);
    if let Some(stronghold) = stronghold_map.get(path) {
        cb(stronghold)
    } else {
        panic!("should initialize stronghold instance before using it")
    }
}

#[cfg(test)]
mod test_utils {
    use super::account::Account;
    use super::account_manager::AccountManager;
    use super::address::{Address, IotaAddress};
    use super::client::ClientOptionsBuilder;
    use super::message::Message;

    use chrono::prelude::Utc;
    use iota::message::prelude::{
        Ed25519Address, Ed25519Signature, MessageId, Payload, SignatureLockedSingleOutput,
        SignatureUnlock, TransactionBuilder, TransactionEssence, TransactionId, UTXOInput,
        UnlockBlock,
    };
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::path::PathBuf;

    use std::convert::TryInto;
    use std::num::NonZeroU64;

    pub fn get_account_manager() -> AccountManager {
        let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
        let storage_path = PathBuf::from(format!("./example-database/{}", storage_path));

        let manager = AccountManager::with_storage_path(storage_path).unwrap();
        manager.set_stronghold_password("password").unwrap();
        manager
    }

    pub fn create_account(
        manager: &AccountManager,
        addresses: Vec<Address>,
        messages: Vec<Message>,
    ) -> Account {
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        manager
            .create_account(client_options)
            .alias("alias")
            .messages(messages)
            .addresses(addresses)
            .initialise()
            .expect("failed to add account")
    }

    pub fn generate_random_iota_address() -> IotaAddress {
        IotaAddress::Ed25519(Ed25519Address::new(rand::random::<[u8; 32]>()))
    }

    pub fn generate_message(
        value: u64,
        address: Address,
        confirmed: bool,
        broadcasted: bool,
    ) -> Message {
        Message {
            id: MessageId::new([0; 32]),
            version: 1,
            trunk: MessageId::new([0; 32]),
            branch: MessageId::new([0; 32]),
            payload_length: 0,
            payload: Payload::Transaction(Box::new(
                TransactionBuilder::new()
                    .with_essence(
                        TransactionEssence::builder()
                            .add_output(
                                SignatureLockedSingleOutput::new(
                                    address.address().clone(),
                                    NonZeroU64::new(value.try_into().unwrap()).unwrap(),
                                )
                                .into(),
                            )
                            .add_input(
                                UTXOInput::new(TransactionId::new([0; 32]), 0)
                                    .unwrap()
                                    .into(),
                            )
                            .finish()
                            .unwrap(),
                    )
                    .add_unlock_block(UnlockBlock::Signature(SignatureUnlock::Ed25519(
                        Ed25519Signature::new([0; 32], Box::new([0])),
                    )))
                    .finish()
                    .unwrap(),
            )),
            timestamp: Utc::now(),
            nonce: 0,
            confirmed,
            broadcasted,
        }
    }
}

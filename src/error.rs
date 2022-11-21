// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Display};

use serde::{ser::Serializer, Serialize};

/// The wallet error type.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", content = "error", rename_all = "camelCase")]
pub enum Error {
    /// Account alias must be unique.
    #[error("can't create account: account alias {0} already exists")]
    AccountAliasAlreadyExists(String),
    /// Account not found
    #[error("account {0} not found")]
    AccountNotFound(String),
    /// Address not found in account
    #[error("address {0} not found in account")]
    AddressNotFoundInAccount(String),
    /// Errors during backup creation or restoring
    #[error("backup failed {0}")]
    BackupError(&'static str),
    /// Error from block crate.
    #[error("{0}")]
    #[serde(serialize_with = "display_string")]
    Block(Box<iota_client::block::Error>),
    /// Block dtos error
    #[error("{0}")]
    #[serde(serialize_with = "display_string")]
    BlockDtoError(#[from] iota_client::block::DtoError),
    /// Burning or melting failed
    #[error("burning or melting failed: {0}")]
    BurningOrMeltingFailed(String),
    /// Client error.
    #[error("`{0}`")]
    #[serde(serialize_with = "display_string")]
    Client(Box<iota_client::Error>),
    /// Funds are spread over too many outputs
    #[error("funds are spread over too many outputs {0}/{1}, consolidation required")]
    ConsolidationRequired(usize, u16),
    /// Crypto.rs error
    #[error("{0}")]
    #[serde(serialize_with = "display_string")]
    CryptoError(#[from] crypto::Error),
    /// Custom input error
    #[error("custom input error {0}")]
    CustomInputError(String),
    /// Failed to get remainder
    #[error("failed to get remainder address")]
    FailedToGetRemainder,
    /// Insufficient funds to send transaction.
    #[error("insufficient funds {0}/{1} available")]
    InsufficientFunds(u64, u64),
    /// Invalid coin type, all accounts need to have the same coin type
    #[error("invalid coin type for new account: {0}, existing coin type is: {1}")]
    InvalidCoinType(u32, u32),
    /// Invalid mnemonic error
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    /// Invalid output kind.
    #[error("invalid output kind: {0}")]
    InvalidOutputKind(String),
    /// IO error. (storage, backup, restore)
    #[error("`{0}`")]
    #[serde(serialize_with = "display_string")]
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
    #[serde(serialize_with = "display_string")]
    JsonError(#[from] serde_json::error::Error),
    /// Minting failed
    #[error("minting failed {0}")]
    MintingFailed(String),
    /// Missing parameter.
    #[error("missing parameter: {0}")]
    MissingParameter(&'static str),
    /// Nft not found in unspent outputs
    #[error("nft not found in unspent outputs")]
    NftNotFoundInUnspentOutputs,
    /// Voting error
    #[cfg(feature = "participation")]
    #[error("voting error {0}")]
    Voting(String),
    /// No outputs available for consolidating
    #[error(
        "nothing to consolidate: available outputs: {available_outputs}, consolidation threshold: {consolidation_threshold}"
    )]
    NoOutputsToConsolidate {
        /// The available outputs for consolidation.
        available_outputs: usize,
        /// The consolidation threshold.
        consolidation_threshold: usize,
    },
    /// Record not found
    #[error("record {0} not found")]
    RecordNotFound(String),
    /// Storage access error.
    #[error("error accessing storage: {0}")]
    Storage(String),
    /// Can't use AccountManager API because the storage is encrypted
    #[error(
        "can't perform operation while storage is encrypted; use AccountManager::set_storage_password to decrypt storage"
    )]
    StorageIsEncrypted,
    /// Tokio task join error
    #[error("{0}")]
    #[serde(serialize_with = "display_string")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

/// Use this to serialize Error variants that implements Debug but not Serialize
fn display_string<T, S>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    value.to_string().serialize(serializer)
}

impl From<iota_client::block::Error> for Error {
    fn from(error: iota_client::block::Error) -> Self {
        Self::Block(Box::new(error))
    }
}

impl From<iota_client::Error> for Error {
    fn from(error: iota_client::Error) -> Self {
        Self::Client(Box::new(error))
    }
}

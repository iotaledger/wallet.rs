// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::ser::{SerializeStruct, Serializer};

/// The wallet error type.
#[derive(Debug, thiserror::Error)]
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
    Block(iota_client::block::Error),
    /// Block dtos error
    #[error("{0}")]
    BlockDtoError(#[from] iota_client::block::DtoError),
    /// Burning or melting failed
    #[error("burning or melting failed: {0}")]
    BurningOrMeltingFailed(String),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(Box<iota_client::Error>),
    /// Funds are spread over too many outputs
    #[error("funds are spread over too many outputs {0}/{1}, consolidation required")]
    ConsolidationRequired(usize, u16),
    /// Crypto.rs error
    #[error("{0}")]
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
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
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
    TaskJoinError(#[from] tokio::task::JoinError),
}

impl From<iota_client::block::Error> for Error {
    fn from(error: iota_client::block::Error) -> Self {
        Self::Block(error)
    }
}

impl From<iota_client::Error> for Error {
    fn from(error: iota_client::Error) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn serialize_variant<S: Serializer>(
            error: &Error,
            serializer: S,
            variant_name: &str,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_struct("Error", 2)?;
            state.serialize_field("type", variant_name)?;
            state.serialize_field("error", &error.to_string())?;
            state.end()
        }

        match self {
            Self::AccountAliasAlreadyExists(_) => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
            Self::AccountNotFound(_) => serialize_variant(self, serializer, "AccountNotFound"),
            Self::AddressNotFoundInAccount(_) => serialize_variant(self, serializer, "AddressNotFoundInAccount"),
            Self::BackupError(_) => serialize_variant(self, serializer, "BackupError"),
            Self::Block(_) => serialize_variant(self, serializer, "Block"),
            Self::BlockDtoError(_) => serialize_variant(self, serializer, "BlockDtoError"),
            Self::BurningOrMeltingFailed(_) => serialize_variant(self, serializer, "BurningOrMeltingFailed"),
            Self::ClientError(_) => serialize_variant(self, serializer, "ClientError"),
            Self::ConsolidationRequired(..) => serialize_variant(self, serializer, "ConsolidationRequired"),
            Self::CryptoError(..) => serialize_variant(self, serializer, "CryptoError"),
            Self::CustomInputError(_) => serialize_variant(self, serializer, "CustomInputError"),
            Self::FailedToGetRemainder => serialize_variant(self, serializer, "FailedToGetRemainder"),
            Self::InsufficientFunds(..) => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::InvalidCoinType(..) => serialize_variant(self, serializer, "InvalidCoinType"),
            Self::InvalidMnemonic(_) => serialize_variant(self, serializer, "InvalidMnemonic"),
            Self::InvalidOutputKind(_) => serialize_variant(self, serializer, "InvalidOutputKind"),
            Self::IoError(_) => serialize_variant(self, serializer, "IoError"),
            Self::JsonError(_) => serialize_variant(self, serializer, "JsonError"),
            Self::MintingFailed(_) => serialize_variant(self, serializer, "MintingFailed"),
            Self::MissingParameter(_) => serialize_variant(self, serializer, "MissingParameter"),
            Self::NftNotFoundInUnspentOutputs => serialize_variant(self, serializer, "NftNotFoundInUnspentOutputs"),
            Self::NoOutputsToConsolidate { .. } => serialize_variant(self, serializer, "NoOutputsToConsolidate"),
            Self::RecordNotFound(_) => serialize_variant(self, serializer, "RecordNotFound"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::StorageIsEncrypted => serialize_variant(self, serializer, "StorageIsEncrypted"),
            Self::TaskJoinError(_) => serialize_variant(self, serializer, "TaskJoinError"),
        }
    }
}

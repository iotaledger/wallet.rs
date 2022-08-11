// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use serde::ser::{SerializeStruct, Serializer};

/// The wallet error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error. (storage, backup, restore)
    #[error("`{0}`")]
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
    JsonError(#[from] serde_json::error::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(Box<iota_client::Error>),
    /// failed to parse address.
    #[error("invalid address")]
    InvalidAddress,
    /// Tried to backup but storage file doesn't exist.
    #[error("storage file doesn't exist")]
    StorageDoesntExist,
    /// Insufficient funds to send transaction.
    #[error("insufficient funds {0}/{1} available")]
    InsufficientFunds(u64, u64),
    /// Latest account is empty (doesn't have history and balance) - can't create account.
    #[error("can't create accounts when the latest account doesn't have block history and balance")]
    LatestAccountIsEmpty,
    /// Account not found
    #[error("account not found")]
    AccountNotFound,
    /// Record not found
    #[error("record not found")]
    RecordNotFound,
    /// Storage access error.
    #[error("error accessing storage: {0}")]
    Storage(String),
    /// Panic error.
    #[error("a panic happened: {0}")]
    Panic(String),
    /// Error from block crate.
    #[error("{0}")]
    Block(iota_client::block::Error),
    /// Block dtos error
    #[error("{0}")]
    BlockDtoError(#[from] iota_client::block::DtoError),
    /// Rest api error
    #[error("{0}")]
    RestApiError(#[from] iota_client::api_types::error::Error),
    /// Errors during backup creation or restoring
    #[error("backup failed {0}")]
    BackupError(&'static str),
    /// Invalid mnemonic error
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    /// Invalid coin type, all accounts need to have the same coin type
    #[error("invalid coin type for new account: {0}, existing coin type is: {1}")]
    InvalidCoinType(u32, u32),
    /// Can't import accounts because the storage already exist
    #[error("failed to restore backup: storage file already exists")]
    StorageExists,
    /// Storage adapter not defined for the given storage path.
    #[error(
        "storage adapter not set for path `{0}`; please use the method `with_storage` on the AccountManager builder"
    )]
    StorageAdapterNotSet(PathBuf),
    /// error decrypting stored record using provided encryptionKey
    #[error("failed to decrypt record: {0}")]
    RecordDecrypt(String),
    /// error encrypting stored record using provided encryptionKey
    #[error("failed to encrypt record: {0}")]
    RecordEncrypt(String),
    /// Can't use AccountManager API because the storage is encrypted
    #[error(
        "can't perform operation while storage is encrypted; use AccountManager::set_storage_password to decrypt storage"
    )]
    StorageIsEncrypted,
    /// Account alias must be unique.
    #[error("can't create account: account alias already exists")]
    AccountAliasAlreadyExists,
    /// Invalid output kind.
    #[error("invalid output kind: {0}")]
    InvalidOutputKind(String),
    /// Missing parameter.
    #[error("missing parameter: {0}")]
    MissingParameter(&'static str),
    /// Failed to get remainder
    #[error("failed to get remainder address")]
    FailedToGetRemainder,
    /// Too many outputs
    #[error("too many outputs: {0}, max is {1}")]
    TooManyOutputs(usize, u16),
    /// Too many outputs
    #[error("too many outputs: {0}, max is {1}")]
    TooManyInputs(usize, u16),
    /// Funds are spread over too many outputs
    #[error("funds are spread over too many outputs {0}/{1}, consolidation required")]
    ConsolidationRequired(usize, u16),
    /// Address not found in account
    #[error("address {0} not found in account")]
    AddressNotFoundInAccount(String),
    /// Minting failed
    #[error("minting failed {0}")]
    MintingFailed(String),
    /// Burning or melting failed
    #[error("burning or melting failed {0}")]
    BurningOrMeltingFailed(String),
    /// Nft not found in unspent outputs
    #[error("nft not found in unspent outputs")]
    NftNotFoundInUnspentOutputs,
    /// Tokio task join error
    #[error("{0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    /// std thread join error
    #[error("thread join error")]
    StdThreadJoinError,
    /// Blake2b256 Error
    #[error("{0}")]
    Blake2b256(&'static str),
    /// Custom input error
    #[error("custom input error {0}")]
    CustomInputError(String),
    /// Client not set error
    #[error("client not set")]
    ClientNotSet,
    /// Local time doesn't match the time of the latest timestamp
    #[error("local time {0} doesn't match the time of the latest timestamp: {1}")]
    TimeNotSynced(u32, u32),
}

impl From<iota_client::Error> for Error {
    fn from(error: iota_client::Error) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<iota_client::block::Error> for Error {
    fn from(error: iota_client::block::Error) -> Self {
        Self::Block(error)
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
            Self::IoError(_) => serialize_variant(self, serializer, "IoError"),
            Self::JsonError(_) => serialize_variant(self, serializer, "JsonError"),
            Self::ClientError(_) => serialize_variant(self, serializer, "ClientError"),
            Self::InvalidAddress => serialize_variant(self, serializer, "InvalidAddress"),
            Self::StorageDoesntExist => serialize_variant(self, serializer, "StorageDoesntExist"),
            Self::InsufficientFunds(..) => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::AccountNotFound => serialize_variant(self, serializer, "AccountNotFound"),
            Self::RecordNotFound => serialize_variant(self, serializer, "RecordNotFound"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::Panic(_) => serialize_variant(self, serializer, "Panic"),
            Self::Block(_) => serialize_variant(self, serializer, "Block"),
            Self::BlockDtoError(_) => serialize_variant(self, serializer, "BlockDtoError"),
            Self::RestApiError(_) => serialize_variant(self, serializer, "RestApiError"),
            Self::InvalidMnemonic(_) => serialize_variant(self, serializer, "InvalidMnemonic"),
            Self::InvalidCoinType(..) => serialize_variant(self, serializer, "InvalidCoinType"),
            Self::BackupError(_) => serialize_variant(self, serializer, "BackupError"),
            Self::StorageExists => serialize_variant(self, serializer, "StorageExists"),
            Self::StorageAdapterNotSet(_) => serialize_variant(self, serializer, "StorageAdapterNotSet"),
            Self::RecordDecrypt(_) => serialize_variant(self, serializer, "RecordDecrypt"),
            Self::RecordEncrypt(_) => serialize_variant(self, serializer, "RecordEncrypt"),
            Self::StorageIsEncrypted => serialize_variant(self, serializer, "StorageIsEncrypted"),
            Self::AccountAliasAlreadyExists => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
            Self::InvalidOutputKind(_) => serialize_variant(self, serializer, "InvalidOutputKind"),
            Self::MissingParameter(_) => serialize_variant(self, serializer, "MissingParameter"),
            Self::FailedToGetRemainder => serialize_variant(self, serializer, "FailedToGetRemainder"),
            Self::TooManyOutputs(..) => serialize_variant(self, serializer, "TooManyOutputs"),
            Self::TooManyInputs(..) => serialize_variant(self, serializer, "TooManyInputs"),
            Self::ConsolidationRequired(..) => serialize_variant(self, serializer, "ConsolidationRequired"),
            Self::AddressNotFoundInAccount(_) => serialize_variant(self, serializer, "AddressNotFoundInAccount"),
            Self::MintingFailed(_) => serialize_variant(self, serializer, "MintingFailed"),
            Self::BurningOrMeltingFailed(_) => serialize_variant(self, serializer, "BurningOrMeltingFailed"),
            Self::NftNotFoundInUnspentOutputs => serialize_variant(self, serializer, "NftNotFoundInUnspentOutputs"),
            Self::TaskJoinError(_) => serialize_variant(self, serializer, "TaskJoinError"),
            Self::StdThreadJoinError => serialize_variant(self, serializer, "StdThreadJoinError"),
            Self::Blake2b256(_) => serialize_variant(self, serializer, "Blake2b256"),
            Self::CustomInputError(_) => serialize_variant(self, serializer, "CustomInputError"),
            Self::ClientNotSet => serialize_variant(self, serializer, "ClientNotSet"),
            Self::TimeNotSynced(..) => serialize_variant(self, serializer, "TimeNotSynced"),
        }
    }
}

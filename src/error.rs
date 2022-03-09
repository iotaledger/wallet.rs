// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::ser::{SerializeStruct, Serializer};

use std::path::PathBuf;

/// The wallet error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error. (storage, backup, restore)
    #[error("`{0}`")]
    IoError(#[from] std::io::Error),
    /// serde_json error.
    #[error("`{0}`")]
    JsonError(#[from] serde_json::error::Error),
    /// stronghold client error.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    #[error("`{0}`")]
    StrongholdError(crate::stronghold::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(Box<iota_client::Error>),
    /// Message not found.
    #[error("message not found")]
    MessageNotFound,
    /// Message id length response invalid.
    #[error("unexpected message_id length")]
    InvalidMessageIdLength,
    /// failed to parse address.
    #[error("invalid address")]
    InvalidAddress,
    /// Address length response invalid.
    #[error("invalid address length")]
    InvalidAddressLength,
    /// Tried to backup but storage file doesn't exist.
    #[error("storage file doesn't exist")]
    StorageDoesntExist,
    /// Insufficient funds to send transfer.
    #[error("insufficient funds {0}/{1} available")]
    InsufficientFunds(u64, u64),
    /// Account isn't empty (has history or balance) - can't delete account.
    #[error("can't delete account: account has history or balance")]
    AccountNotEmpty,
    /// Latest account is empty (doesn't have history and balance) - can't create account.
    #[error("can't create accounts when the latest account doesn't have message history and balance")]
    LatestAccountIsEmpty,
    /// Account not found
    #[error("account not found")]
    AccountNotFound,
    /// Record not found
    #[error("Record not found")]
    RecordNotFound,
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
    /// Error from bee_message crate.
    #[error("{0}")]
    BeeMessage(iota_client::bee_message::Error),
    /// Bee rest api error
    #[error("{0}")]
    BeeRestApiError(#[from] iota_client::bee_rest_api::types::error::Error),
    /// Path provided to `import_accounts` isn't a valid file
    #[error("provided backup path isn't a valid file")]
    InvalidBackupFile,
    /// Backup `destination` argument is invalid
    #[error("backup destination must be an existing directory or a file on an existing directory")]
    InvalidBackupDestination,
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
    #[error("address not found in account")]
    AddressNotFoundInAccount,
    /// Tokio task join error
    #[error("{0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    /// std thread join error
    #[error("Thread join error")]
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
    /// Error from the logger in the bee_common crate.
    #[error("{0}")]
    BeeCommonLogger(iota_client::common::logger::Error),
    /// Local time doesn't match the time of the latest timestamp
    #[error("Local time {0} doesn't match the time of the latest timestamp: {1}")]
    TimeNotSynced(u64, u64),
}

impl From<iota_client::Error> for Error {
    fn from(error: iota_client::Error) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<iota_client::bee_message::Error> for Error {
    fn from(error: iota_client::bee_message::Error) -> Self {
        Self::BeeMessage(error)
    }
}

impl From<iota_client::common::logger::Error> for Error {
    fn from(error: iota_client::common::logger::Error) -> Self {
        Self::BeeCommonLogger(error)
    }
}

#[cfg(feature = "stronghold")]
impl From<crate::stronghold::Error> for Error {
    fn from(error: crate::stronghold::Error) -> Self {
        match error {
            crate::stronghold::Error::RecordNotFound => Self::RecordNotFound,
            _ => Self::StrongholdError(error),
        }
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
            #[cfg(feature = "stronghold")]
            Self::StrongholdError(_) => serialize_variant(self, serializer, "StrongholdError"),
            Self::ClientError(_) => serialize_variant(self, serializer, "ClientError"),
            Self::MessageNotFound => serialize_variant(self, serializer, "MessageNotFound"),
            Self::InvalidMessageIdLength => serialize_variant(self, serializer, "InvalidMessageIdLength"),
            Self::InvalidAddress => serialize_variant(self, serializer, "InvalidAddress"),
            Self::InvalidAddressLength => serialize_variant(self, serializer, "InvalidAddressLength"),
            Self::StorageDoesntExist => serialize_variant(self, serializer, "StorageDoesntExist"),
            Self::InsufficientFunds(..) => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::AccountNotEmpty => serialize_variant(self, serializer, "AccountNotEmpty"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::AccountNotFound => serialize_variant(self, serializer, "AccountNotFound"),
            Self::RecordNotFound => serialize_variant(self, serializer, "RecordNotFound"),
            Self::InvalidRemainderValueAddress => serialize_variant(self, serializer, "InvalidRemainderValueAddress"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::Panic(_) => serialize_variant(self, serializer, "Panic"),
            Self::BeeMessage(_) => serialize_variant(self, serializer, "BeeMessage"),
            Self::BeeRestApiError(_) => serialize_variant(self, serializer, "BeeRestApiError"),
            Self::InvalidMnemonic(_) => serialize_variant(self, serializer, "InvalidMnemonic"),
            Self::InvalidBackupFile => serialize_variant(self, serializer, "InvalidBackupFile"),
            Self::InvalidBackupDestination => serialize_variant(self, serializer, "InvalidBackupDestination"),
            Self::StorageExists => serialize_variant(self, serializer, "StorageExists"),
            Self::StorageAdapterNotSet(_) => serialize_variant(self, serializer, "StorageAdapterNotSet"),
            Self::RecordDecrypt(_) => serialize_variant(self, serializer, "RecordDecrypt"),
            Self::RecordEncrypt(_) => serialize_variant(self, serializer, "RecordEncrypt"),
            Self::StorageIsEncrypted => serialize_variant(self, serializer, "StorageIsEncrypted"),
            Self::AccountAliasAlreadyExists => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
            Self::InvalidOutputKind(_) => serialize_variant(self, serializer, "InvalidOutputKind"),
            Self::FailedToGetRemainder => serialize_variant(self, serializer, "FailedToGetRemainder"),
            Self::TooManyOutputs(..) => serialize_variant(self, serializer, "TooManyOutputs"),
            Self::TooManyInputs(..) => serialize_variant(self, serializer, "TooManyInputs"),
            Self::ConsolidationRequired(..) => serialize_variant(self, serializer, "ConsolidationRequired"),
            Self::AddressNotFoundInAccount => serialize_variant(self, serializer, "AddressNotFoundInAccount"),
            Self::TaskJoinError(_) => serialize_variant(self, serializer, "TaskJoinError"),
            Self::StdThreadJoinError => serialize_variant(self, serializer, "StdThreadJoinError"),
            Self::Blake2b256(_) => serialize_variant(self, serializer, "Blake2b256"),
            Self::CustomInputError(_) => serialize_variant(self, serializer, "CustomInputError"),
            Self::ClientNotSet => serialize_variant(self, serializer, "ClientNotSet"),
            Self::BeeCommonLogger(_) => serialize_variant(self, serializer, "BeeCommonLogger"),
            Self::TimeNotSynced(..) => serialize_variant(self, serializer, "TimeNotSynced"),
        }
    }
}

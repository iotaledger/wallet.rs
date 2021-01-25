// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::ser::{SerializeStruct, Serializer};
use std::path::PathBuf;

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
    #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
    #[error("`{0}`")]
    StrongholdError(crate::stronghold::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(#[from] iota::client::Error),
    /// url parse error (client options builder).
    #[error("`{0}`")]
    UrlError(#[from] url::ParseError),
    /// Message not found.
    #[error("message not found")]
    MessageNotFound,
    /// Message id length response invalid.
    #[error("unexpected message_id length")]
    InvalidMessageIdLength,
    /// failed to parse address.
    #[error("invalid address")]
    InvalidAddress,
    /// Tried to backup but storage file doesn't exist.
    #[error("storage file doesn't exist")]
    StorageDoesntExist,
    /// Insufficient funds to send transfer.
    #[error("insufficient funds")]
    InsufficientFunds,
    /// Account isn't empty (has history or balance) - can't delete account.
    #[error("can't delete account: account has history or balance")]
    AccountNotEmpty,
    /// Latest account is empty (doesn't have history and balance) - can't create account.
    #[error("can't create accounts when the latest account doesn't have message history and balance")]
    LatestAccountIsEmpty,
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
    InternalTransferDestinationEmpty,
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
    /// Error from bee_message crate.
    #[error("{0}")]
    BeeMessage(iota::message::Error),
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
    #[error("mnemonic encode error: {0}")]
    MnemonicEncode(String),
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
    #[error("failed to decrypt account: {0}")]
    AccountDecrypt(String),
    /// error encrypting stored account using provided encryptionKey
    #[error("failed to encrypt account: {0}")]
    AccountEncrypt(String),
    /// Can't use AccountManager API because the storage is encrypted
    #[error(
        "can't perform operation while storage is encrypted; use AccountManager::set_storage_password to decrypt storage"
    )]
    StorageIsEncrypted,
    /// cannot use index to get account - multiple index sequences found (two or more different signer types stored on
    /// accounts)
    #[error("cannot use index identifier when two signer types are used")]
    CannotUseIndexIdentifier,
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
    /// Ledger Essence Too Large
    #[error("ledger essence too large")]
    LedgerEssenceTooLarge,
    /// Account alias must be unique.
    #[error("can't create account: account alias already exists")]
    AccountAliasAlreadyExists,
}

impl Drop for Error {
    fn drop(&mut self) {
        crate::event::emit_error(self);
    }
}

impl From<iota::message::Error> for Error {
    fn from(error: iota::message::Error) -> Self {
        Self::BeeMessage(error)
    }
}

#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
impl From<crate::stronghold::Error> for Error {
    fn from(error: crate::stronghold::Error) -> Self {
        match error {
            crate::stronghold::Error::AccountNotFound => Self::AccountNotFound,
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
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            Self::StrongholdError(_) => serialize_variant(self, serializer, "StrongholdError"),
            Self::ClientError(_) => serialize_variant(self, serializer, "ClientError"),
            Self::UrlError(_) => serialize_variant(self, serializer, "UrlError"),
            Self::MessageNotFound => serialize_variant(self, serializer, "MessageNotFound"),
            Self::InvalidMessageIdLength => serialize_variant(self, serializer, "InvalidMessageIdLength"),
            Self::InvalidAddress => serialize_variant(self, serializer, "InvalidAddress"),
            Self::StorageDoesntExist => serialize_variant(self, serializer, "StorageDoesntExist"),
            Self::InsufficientFunds => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::AccountNotEmpty => serialize_variant(self, serializer, "AccountNotEmpty"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::AccountNotFound => serialize_variant(self, serializer, "AccountNotFound"),
            Self::InvalidRemainderValueAddress => serialize_variant(self, serializer, "InvalidRemainderValueAddress"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::Panic(_) => serialize_variant(self, serializer, "Panic"),
            Self::InternalTransferDestinationEmpty => {
                serialize_variant(self, serializer, "InternalTransferDestinationEmpty")
            }
            Self::InvalidMessageId => serialize_variant(self, serializer, "InvalidMessageId"),
            Self::InvalidTransactionId => serialize_variant(self, serializer, "InvalidTransactionId"),
            Self::AddressBuildRequiredField(_) => serialize_variant(self, serializer, "AddressBuildRequiredField"),
            Self::AccountInitialiseRequiredField(_) => {
                serialize_variant(self, serializer, "AccountInitialiseRequiredField")
            }
            Self::BeeMessage(_) => serialize_variant(self, serializer, "BeeMessage"),
            Self::MnemonicEncode(_) => serialize_variant(self, serializer, "MnemonicEncode"),
            Self::InvalidMnemonic(_) => serialize_variant(self, serializer, "InvalidMnemonic"),
            Self::InvalidBackupFile => serialize_variant(self, serializer, "InvalidBackupFile"),
            Self::InvalidBackupDestination => serialize_variant(self, serializer, "InvalidBackupDestination"),
            Self::StorageAdapterNotDefined => serialize_variant(self, serializer, "StorageAdapterNotDefined"),
            Self::StorageExists => serialize_variant(self, serializer, "StorageExists"),
            Self::StorageAdapterNotSet(_) => serialize_variant(self, serializer, "StorageAdapterNotSet"),
            Self::AccountDecrypt(_) => serialize_variant(self, serializer, "AccountDecrypt"),
            Self::AccountEncrypt(_) => serialize_variant(self, serializer, "AccountEncrypt"),
            Self::StorageIsEncrypted => serialize_variant(self, serializer, "StorageIsEncrypted"),
            Self::CannotUseIndexIdentifier => serialize_variant(self, serializer, "CannotUseIndexIdentifier"),
            Self::LedgerMiscError => serialize_variant(self, serializer, "LedgerMiscError"),
            Self::LedgerDongleLocked => serialize_variant(self, serializer, "LedgerDongleLocked"),
            Self::LedgerDeniedByUser => serialize_variant(self, serializer, "LedgerDeniedByUser"),
            Self::LedgerDeviceNotFound => serialize_variant(self, serializer, "LedgerDeviceNotFound"),
            Self::LedgerEssenceTooLarge => serialize_variant(self, serializer, "LedgerEssenceTooLarge"),
            Self::AccountAliasAlreadyExists => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
        }
    }
}

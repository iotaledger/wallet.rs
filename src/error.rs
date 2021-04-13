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
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    #[error("`{0}`")]
    StrongholdError(crate::stronghold::Error),
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(Box<iota::client::Error>),
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
    /// Address length response invalid.
    #[error("invalid address length")]
    InvalidAddressLength,
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
    #[error("backup destination must be an existing directory or a file on an existing directory")]
    InvalidBackupDestination,
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
    /// cannot use index to get account - multiple index sequences found (two or more different signer types stored on
    /// accounts)
    #[error("cannot use index identifier when two signer types are used")]
    CannotUseIndexIdentifier,
    /// Ledger transport error
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger transport error")]
    LedgerMiscError,
    /// Dongle Locked
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger locked")]
    LedgerDongleLocked,
    /// Denied by User
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("denied by user")]
    LedgerDeniedByUser,
    /// Ledger Device not found
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger device not found")]
    LedgerDeviceNotFound,
    /// Ledger Essence Too Large
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger essence too large")]
    LedgerEssenceTooLarge,
    /// Account alias must be unique.
    #[error("can't create account: account alias already exists")]
    AccountAliasAlreadyExists,
    /// Dust error, for example not enough balance on an address.
    #[error("Dust error: {0}")]
    DustError(String),
    /// Invalid output kind.
    #[error("invalid output kind: {0}")]
    InvalidOutputKind(String),
    /// Node not synced when creating account or updating client options.
    #[error("nodes {0} not synced")]
    NodesNotSynced(String),
    /// iota 1.0 client error
    // #[cfg(feature = "migration")]
    #[error(transparent)]
    LegacyClientError(Box<iota_migration::client::Error>),
    /// Invalid legacy seed.
    #[error("invalid seed")]
    InvalidSeed,
    /// Migration data not found.
    #[error("migration data not found for the provided seed; call `get_migration_data` first.")]
    MigrationDataNotFound,
    /// Migration bundle not found.
    #[error("migration bundle not found with the provided bundle hash")]
    MigrationBundleNotFound,
    /// Input not found with given index.
    #[error("input not found with the provided index")]
    InputNotFound,
    /// Empty input list on migration bundle creation.
    #[error("can't create migration bundle: input list is empty")]
    EmptyInputList,
    /// Cannot create bundle when the number of inputs is larger than 1 and there's a spent input.
    /// Spent addresses must be the only input in a bundle.
    #[error("can't create migration bundle: the bundle has more than one input and one of them are spent")]
    SpentAddressOnBundle,
}

impl Drop for Error {
    fn drop(&mut self) {
        crate::event::emit_error(self);
    }
}

impl From<iota::client::Error> for Error {
    fn from(error: iota::client::Error) -> Self {
        Self::ClientError(Box::new(error))
    }
}

impl From<iota_migration::client::Error> for Error {
    fn from(error: iota_migration::client::Error) -> Self {
        Self::LegacyClientError(Box::new(error))
    }
}

impl From<iota::message::Error> for Error {
    fn from(error: iota::message::Error) -> Self {
        Self::BeeMessage(error)
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

// map most errors to a single error but there are some errors that
// need special care.
// LedgerDongleLocked: Ask the user to unlock the dongle
// LedgerDeniedByUser: The user denied a signing
// LedgerDeviceNotFound: No usable Ledger device was found
// LedgerMiscError: Everything else.
// LedgerEssenceTooLarge: Essence with bip32 input indices need more space then the internal buffer is big
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
impl From<iota_ledger::api::errors::APIError> for Error {
    fn from(error: iota_ledger::api::errors::APIError) -> Self {
        log::info!("ledger error: {}", error);
        match error {
            iota_ledger::api::errors::APIError::SecurityStatusNotSatisfied => Error::LedgerDongleLocked,
            iota_ledger::api::errors::APIError::ConditionsOfUseNotSatisfied => Error::LedgerDeniedByUser,
            iota_ledger::api::errors::APIError::TransportError => Error::LedgerDeviceNotFound,
            iota_ledger::api::errors::APIError::EssenceTooLarge => Error::LedgerEssenceTooLarge,
            _ => Error::LedgerMiscError,
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
            Self::UrlError(_) => serialize_variant(self, serializer, "UrlError"),
            Self::MessageNotFound => serialize_variant(self, serializer, "MessageNotFound"),
            Self::InvalidMessageIdLength => serialize_variant(self, serializer, "InvalidMessageIdLength"),
            Self::InvalidAddress => serialize_variant(self, serializer, "InvalidAddress"),
            Self::InvalidAddressLength => serialize_variant(self, serializer, "InvalidAddressLength"),
            Self::StorageDoesntExist => serialize_variant(self, serializer, "StorageDoesntExist"),
            Self::InsufficientFunds => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::AccountNotEmpty => serialize_variant(self, serializer, "AccountNotEmpty"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::RecordNotFound => serialize_variant(self, serializer, "RecordNotFound"),
            Self::InvalidRemainderValueAddress => serialize_variant(self, serializer, "InvalidRemainderValueAddress"),
            Self::Storage(_) => serialize_variant(self, serializer, "Storage"),
            Self::Panic(_) => serialize_variant(self, serializer, "Panic"),
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
            Self::StorageExists => serialize_variant(self, serializer, "StorageExists"),
            Self::StorageAdapterNotSet(_) => serialize_variant(self, serializer, "StorageAdapterNotSet"),
            Self::RecordDecrypt(_) => serialize_variant(self, serializer, "RecordDecrypt"),
            Self::RecordEncrypt(_) => serialize_variant(self, serializer, "RecordEncrypt"),
            Self::StorageIsEncrypted => serialize_variant(self, serializer, "StorageIsEncrypted"),
            Self::CannotUseIndexIdentifier => serialize_variant(self, serializer, "CannotUseIndexIdentifier"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerMiscError => serialize_variant(self, serializer, "LedgerMiscError"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerDongleLocked => serialize_variant(self, serializer, "LedgerDongleLocked"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerDeniedByUser => serialize_variant(self, serializer, "LedgerDeniedByUser"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerDeviceNotFound => serialize_variant(self, serializer, "LedgerDeviceNotFound"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerEssenceTooLarge => serialize_variant(self, serializer, "LedgerEssenceTooLarge"),
            Self::AccountAliasAlreadyExists => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
            Self::DustError(_) => serialize_variant(self, serializer, "DustError"),
            Self::InvalidOutputKind(_) => serialize_variant(self, serializer, "InvalidOutputKind"),
            Self::NodesNotSynced(_) => serialize_variant(self, serializer, "NodesNotSynced"),
            // #[cfg(feature = "migration")]
            Self::LegacyClientError(_) => serialize_variant(self, serializer, "LegacyClientError"),
            Self::InvalidSeed => serialize_variant(self, serializer, "InvalidSeed"),
            Self::MigrationDataNotFound => serialize_variant(self, serializer, "MigrationDataNotFound"),
            Self::MigrationBundleNotFound => serialize_variant(self, serializer, "MigrationBundleNotFound"),
            Self::InputNotFound => serialize_variant(self, serializer, "InputNotFound"),
            Self::EmptyInputList => serialize_variant(self, serializer, "EmptyInputList"),
            Self::SpentAddressOnBundle => serialize_variant(self, serializer, "SpentAddressOnBundle"),
        }
    }
}

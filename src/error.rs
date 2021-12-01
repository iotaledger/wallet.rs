// Copyright 2021 IOTA Stiftung
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
    ClientError(Box<iota_client::Error>),
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
    #[error("insufficient funds {0}/{1} available or input address used as output")]
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
    BeeMessage(iota_client::bee_message::Error),
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
    /// No ledger signer error
    #[error("no ledger signer")]
    NoLedgerSignerError,
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
    /// Ledger transport error
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger app compiled for testnet but used with mainnet or vice versa")]
    LedgerNetMismatch,
    /// Wrong ledger seed error
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[error("ledger mnemonic is mismatched")]
    LedgerMnemonicMismatch,
    /// Account alias must be unique.
    #[error("can't create account: account alias already exists")]
    AccountAliasAlreadyExists,
    /// Dust error, for example no dust allowance ouptut on an address.
    #[error("Dust error: {0}")]
    DustError(String),
    /// Leaving dust error, for example sending 2.5 from 3 Mi, would leave 0.5 (dust) behind.
    #[error("Leaving dust error: {0}")]
    LeavingDustError(String),
    /// Invalid output kind.
    #[error("invalid output kind: {0}")]
    InvalidOutputKind(String),
    /// Node not synced when creating account or updating client options.
    #[error("nodes {0} not synced")]
    NodesNotSynced(String),
    /// Failed to get remainder
    #[error("failed to get remainder address")]
    FailedToGetRemainder,
    /// Too many outputs
    #[error("too many outputs: {0}, max is {1}")]
    TooManyOutputs(usize, usize),
    /// Too many outputs
    #[error("too many outputs: {0}, max is {1}")]
    TooManyInputs(usize, usize),
    /// Funds are spread over too many outputs
    #[error("funds are spread over too many outputs {0}/{1}, consolidation required")]
    ConsolidationRequired(usize, usize),
    /// Provided input address not found
    #[error("provided input address not found")]
    InputAddressNotFound,
    /// Mutex lock failed.
    #[error("Mutex lock failed")]
    PoisonError,
    /// Tokio task join error
    #[error("{0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
    /// std thread join error
    #[error("Thread join error")]
    StdThreadJoinError,
    /// Couldn't get a spent output from a node.
    #[error("couldn't get a spent output from node")]
    SpentOutputNotFound,
    #[cfg(feature = "mnemonic")]
    /// Blake2b256 Error
    #[error("{0}")]
    Blake2b256(&'static str),
    #[cfg(feature = "mnemonic")]
    #[error("invalid address or account index {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[cfg(feature = "mnemonic")]
    /// Crypto.rs error
    #[error("{0}")]
    Crypto(#[from] crypto::Error),
    #[cfg(feature = "mnemonic")]
    /// Mnemonic not set error
    #[error("mnemonic not set")]
    MnemonicNotSet,
    /// Missing unlock block error
    #[error("missing unlock block")]
    MissingUnlockBlock,
    /// Custom input error
    #[error("custom input error {0}")]
    CustomInputError(String),
    /// Client not set error
    #[error("client not set")]
    ClientNotSet,
    /// Error from the logger in the bee_common crate.
    #[error("{0}")]
    BeeCommonLogger(iota_client::common::logger::Error),
    /// Empty output amount error
    #[error("output amount can't be 0")]
    EmptyOutputAmount,
}

// impl Drop for Error {
//     fn drop(&mut self) {
//         crate::event::emit_error(self);
//     }
// }

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
            Self::InsufficientFunds(_, _) => serialize_variant(self, serializer, "InsufficientFunds"),
            Self::AccountNotEmpty => serialize_variant(self, serializer, "AccountNotEmpty"),
            Self::LatestAccountIsEmpty => serialize_variant(self, serializer, "LatestAccountIsEmpty"),
            Self::AccountNotFound => serialize_variant(self, serializer, "AccountNotFound"),
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
            Self::NoLedgerSignerError => serialize_variant(self, serializer, "NoLedgerSignerError"),
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
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerNetMismatch => serialize_variant(self, serializer, "LedgerNetMismatch"),
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            Self::LedgerMnemonicMismatch => serialize_variant(self, serializer, "LedgerMnemonicMismatch"),
            Self::AccountAliasAlreadyExists => serialize_variant(self, serializer, "AccountAliasAlreadyExists"),
            Self::DustError(_) => serialize_variant(self, serializer, "DustError"),
            Self::LeavingDustError(_) => serialize_variant(self, serializer, "LeavingDustError"),
            Self::InvalidOutputKind(_) => serialize_variant(self, serializer, "InvalidOutputKind"),
            Self::NodesNotSynced(_) => serialize_variant(self, serializer, "NodesNotSynced"),
            Self::FailedToGetRemainder => serialize_variant(self, serializer, "FailedToGetRemainder"),
            Self::TooManyOutputs(_, _) => serialize_variant(self, serializer, "TooManyOutputs"),
            Self::TooManyInputs(_, _) => serialize_variant(self, serializer, "TooManyInputs"),
            Self::ConsolidationRequired(_, _) => serialize_variant(self, serializer, "ConsolidationRequired"),
            Self::InputAddressNotFound => serialize_variant(self, serializer, "InputAddressNotFound"),
            Self::PoisonError => serialize_variant(self, serializer, "PoisonError"),
            Self::TaskJoinError(_) => serialize_variant(self, serializer, "TaskJoinError"),
            Self::StdThreadJoinError => serialize_variant(self, serializer, "StdThreadJoinError"),
            Self::SpentOutputNotFound => serialize_variant(self, serializer, "SpentOutputNotFound"),
            #[cfg(feature = "mnemonic")]
            Self::Blake2b256(_) => serialize_variant(self, serializer, "Blake2b256"),
            #[cfg(feature = "mnemonic")]
            Self::TryFromInt(_) => serialize_variant(self, serializer, "TryFromInt"),
            #[cfg(feature = "mnemonic")]
            Self::Crypto(_) => serialize_variant(self, serializer, "Crypto"),
            #[cfg(feature = "mnemonic")]
            Self::MnemonicNotSet => serialize_variant(self, serializer, "MnemonicNotSet"),
            Self::MissingUnlockBlock => serialize_variant(self, serializer, "MissingUnlockBlock"),
            Self::CustomInputError(_) => serialize_variant(self, serializer, "CustomInputError"),
            Self::ClientNotSet => serialize_variant(self, serializer, "ClientNotSet"),
            Self::BeeCommonLogger(_) => serialize_variant(self, serializer, "BeeCommonLogger"),
            Self::EmptyOutputAmount => serialize_variant(self, serializer, "EmptyOutputAmount"),
        }
    }
}

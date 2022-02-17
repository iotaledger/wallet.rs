// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        operations::{syncing::SyncOptions, transfer::TransferOptions},
        types::AccountIdentifier,
    },
    client::options::ClientOptions,
};

use super::account_method::AccountMethod;
use iota_client::bee_message::output::Output;
use serde::{ser::Serializer, Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

/// An account to create.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountToCreate {
    /// The account alias.
    pub alias: Option<String>,
}

/// The messages that can be sent to the actor.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum MessageType {
    /// Creates an account.
    CreateAccount(Box<AccountToCreate>),
    /// Read account.
    GetAccount(AccountIdentifier),
    /// Read accounts.
    GetAccounts,
    /// Consume an account method.
    CallAccountMethod {
        /// The account identifier.
        account_id: AccountIdentifier,
        /// The account method to call.
        method: AccountMethod,
    },
    #[cfg(feature = "storage")]
    /// Backup storage.
    Backup {
        /// The backup destination.
        destination: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    #[cfg(feature = "storage")]
    /// Import accounts from storage.
    RestoreBackup {
        /// The path to the backed up storage.
        source: String,
        /// Stronghold file password.
        password: String,
    },
    #[cfg(feature = "storage")]
    /// Deletes the storage.
    DeleteStorage,
    /// Send funds.
    SendTransfer {
        /// The account identifier.
        account_id: AccountIdentifier,
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
    },
    /// Generates a new mnemonic.
    GenerateMnemonic,
    /// Checks if the given mnemonic is valid.
    VerifyMnemonic(String),
    /// Updates the client options for all accounts.
    SetClientOptions(Box<ClientOptions>),
    /// Start background syncing.
    StartBackgroundSync {
        /// Sync options
        options: Option<SyncOptions>,
        /// Interval
        interval: Option<Duration>,
    },
    /// Stop background syncing.
    StopBackgroundSync,
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MessageType::CreateAccount(_) => serializer.serialize_unit_variant("MessageType", 1, "CreateAccount"),
            MessageType::GetAccount(_) => serializer.serialize_unit_variant("MessageType", 2, "GetAccount"),
            MessageType::GetAccounts => serializer.serialize_unit_variant("MessageType", 3, "GetAccounts"),
            MessageType::CallAccountMethod { .. } => {
                serializer.serialize_unit_variant("MessageType", 4, "CallAccountMethod")
            }
            MessageType::Backup { .. } => serializer.serialize_unit_variant("MessageType", 7, "Backup"),
            MessageType::RestoreBackup { .. } => serializer.serialize_unit_variant("MessageType", 8, "RestoreBackup"),
            MessageType::SendTransfer { .. } => serializer.serialize_unit_variant("MessageType", 15, "SendTransfer"),
            MessageType::GenerateMnemonic => serializer.serialize_unit_variant("MessageType", 17, "GenerateMnemonic"),
            MessageType::VerifyMnemonic(_) => serializer.serialize_unit_variant("MessageType", 18, "VerifyMnemonic"),
            MessageType::DeleteStorage => serializer.serialize_unit_variant("MessageType", 22, "DeleteStorage"),
            MessageType::SetClientOptions(_) => {
                serializer.serialize_unit_variant("MessageType", 24, "SetClientOptions")
            }
            MessageType::StartBackgroundSync { .. } => {
                serializer.serialize_unit_variant("MessageType", 34, "StartBackgroundSync")
            }
            MessageType::StopBackgroundSync => {
                serializer.serialize_unit_variant("MessageType", 35, "StopBackgroundSync")
            }
        }
    }
}

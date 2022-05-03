// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, time::Duration};

use serde::{ser::Serializer, Deserialize, Serialize};

use super::account_method::AccountMethod;
#[cfg(feature = "events")]
#[cfg(debug_assertions)]
use crate::events::types::WalletEvent;
use crate::{
    account::{operations::syncing::SyncOptions, types::AccountIdentifier},
    ClientOptions,
};

/// An account to create.
#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountToCreate {
    /// The account alias.
    pub alias: Option<String>,
    /// The account coin type.
    pub coin_type: Option<u32>,
}

/// The messages that can be sent to the actor.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    /// Creates an account.
    /// Expected response: [`Account`](crate::message_interface::ResponseType::Account)
    CreateAccount(Box<AccountToCreate>),
    /// Read account.
    /// Expected response: [`Account`](crate::message_interface::ResponseType::Account)
    GetAccount(AccountIdentifier),
    /// Read accounts.
    /// Expected response: [`Accounts`](crate::message_interface::ResponseType::Accounts)
    GetAccounts,
    /// Consume an account method.
    /// Returns [`ResponseType`](crate::message_interface::ResponseType)
    CallAccountMethod {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The account method to call.
        method: AccountMethod,
    },
    /// Backup storage.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    #[cfg(feature = "storage")]
    Backup {
        /// The backup destination.
        destination: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    /// Find accounts with unspent outputs
    RecoverAccounts {
        #[serde(rename = "accountGapLimit")]
        /// Defines how many accounts without unspent outputs will be
        /// checked, if an account has unspent outputs, the counter is reset
        account_gap_limit: u32,
        #[serde(rename = "addressGapLimit")]
        /// Defines how many addresses without unspent outputs will be checked in each account, if an
        /// address has unspent outputs, the counter is reset
        address_gap_limit: u32,
    },
    /// Import accounts from a Stronghold backup.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    #[cfg(feature = "storage")]
    RestoreBackup {
        /// The path to the backed up Stronghold.
        source: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    /// Deletes the storage.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    #[cfg(feature = "storage")]
    DeleteStorage,
    /// Generates a new mnemonic.
    /// Expected response: [`GeneratedMnemonic`](crate::message_interface::ResponseType::GeneratedMnemonic)
    GenerateMnemonic,
    /// Checks if the given mnemonic is valid.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    VerifyMnemonic(String),
    /// Updates the client options for all accounts.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    SetClientOptions(Box<ClientOptions>),
    /// Get the node information
    /// Expected response: [`NodeInfo`](crate::message_interface::ResponseType::NodeInfo)
    GetNodeInfo,
    /// Set the stronghold password.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    SetStrongholdPassword(String),
    /// Store a mnemonic into the Stronghold vault.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    StoreMnemonic(String),
    /// Start background syncing.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    StartBackgroundSync {
        /// Sync options
        options: Option<SyncOptions>,
        /// Interval
        interval: Option<Duration>,
    },
    /// Stop background syncing.
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    StopBackgroundSync,
    /// Emits an event for testing if the event system is working
    /// Expected response: [`Ok`](crate::message_interface::ResponseType::Ok)
    #[cfg(feature = "events")]
    #[cfg(debug_assertions)]
    EmitTestEvent(WalletEvent),
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
            #[cfg(feature = "stronghold")]
            MessageType::Backup { .. } => serializer.serialize_unit_variant("MessageType", 5, "Backup"),
            MessageType::RecoverAccounts { .. } => {
                serializer.serialize_unit_variant("MessageType", 7, "RecoverAccounts")
            #[cfg(feature = "stronghold")]
            MessageType::RestoreBackup { .. } => serializer.serialize_unit_variant("MessageType", 6, "RestoreBackup"),
            MessageType::GenerateMnemonic => serializer.serialize_unit_variant("MessageType", 8, "GenerateMnemonic"),
            MessageType::VerifyMnemonic(_) => serializer.serialize_unit_variant("MessageType", 9, "VerifyMnemonic"),
            MessageType::DeleteStorage => serializer.serialize_unit_variant("MessageType", 10, "DeleteStorage"),
            MessageType::SetClientOptions(_) => {
                serializer.serialize_unit_variant("MessageType", 11, "SetClientOptions")
            }
            MessageType::GetNodeInfo => serializer.serialize_unit_variant("MessageType", 12, "GetNodeInfo"),
            MessageType::SetStrongholdPassword(_) => {
                serializer.serialize_unit_variant("MessageType", 12, "SetStrongholdPassword")
            }
            MessageType::StoreMnemonic(_) => serializer.serialize_unit_variant("MessageType", 13, "StoreMnemonic"),
            MessageType::StartBackgroundSync { .. } => {
                serializer.serialize_unit_variant("MessageType", 14, "StartBackgroundSync")
            }
            MessageType::StopBackgroundSync => {
                serializer.serialize_unit_variant("MessageType", 15, "StopBackgroundSync")
            }
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            MessageType::EmitTestEvent(_) => serializer.serialize_unit_variant("MessageType", 16, "EmitTestEvent"),
        }
    }
}

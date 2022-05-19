// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    fmt::{Debug, Formatter, Result},
    path::PathBuf,
    time::Duration,
};

use iota_client::node_manager::node::NodeAuth;
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
pub struct AccountToCreate {
    /// The account alias.
    pub alias: Option<String>,
    /// The account coin type.
    #[serde(rename = "coinType")]
    pub coin_type: Option<u32>,
}

/// The messages that can be sent to the actor.
#[derive(Clone, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    /// Creates an account.
    /// Expected response: [`Account`](crate::message_interface::Response::Account)
    CreateAccount(Box<AccountToCreate>),
    /// Read account.
    /// Expected response: [`Account`](crate::message_interface::Response::Account)
    GetAccount(AccountIdentifier),
    /// Read accounts.
    /// Expected response: [`Accounts`](crate::message_interface::Response::Accounts)
    GetAccounts,
    /// Consume an account method.
    /// Returns [`Response`](crate::message_interface::Response)
    CallAccountMethod {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The account method to call.
        method: AccountMethod,
    },
    /// Backup storage.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "stronghold")]
    Backup {
        /// The backup destination.
        destination: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    /// Clears the Stronghold password from memory.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "stronghold")]
    ClearStrongholdPassword,
    /// Checks if the Stronghold password is available.
    /// Expected response:
    /// [`StrongholdPasswordIsAvailable`](crate::message_interface::Response::StrongholdPasswordIsAvailable)
    #[cfg(feature = "stronghold")]
    IsStrongholdPasswordAvailable,
    /// Find accounts with unspent outputs
    /// Expected response: [`Accounts`](crate::message_interface::Response::Accounts)
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
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "stronghold")]
    RestoreBackup {
        /// The path to the backed up Stronghold.
        source: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    /// Deletes the storage.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "storage")]
    DeleteStorage,
    /// Generates a new mnemonic.
    /// Expected response: [`GeneratedMnemonic`](crate::message_interface::Response::GeneratedMnemonic)
    GenerateMnemonic,
    /// Checks if the given mnemonic is valid.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    VerifyMnemonic(String),
    /// Updates the client options for all accounts.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetClientOptions(Box<ClientOptions>),
    /// Get the node information
    /// Expected response: [`NodeInfo`](crate::message_interface::Response::NodeInfo)
    GetNodeInfo {
        /// Url
        url: Option<String>,
        /// Node authentication
        auth: Option<NodeAuth>,
    },
    /// Set the stronghold password.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetStrongholdPassword(String),
    /// Store a mnemonic into the Stronghold vault.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    StoreMnemonic(String),
    /// Start background syncing.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    StartBackgroundSync {
        /// Sync options
        options: Option<SyncOptions>,
        /// Interval
        interval: Option<Duration>,
    },
    /// Stop background syncing.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    StopBackgroundSync,
    /// Emits an event for testing if the event system is working
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "events")]
    #[cfg(debug_assertions)]
    EmitTestEvent(WalletEvent),
}

// Custom Debug implementation to not log secrets
impl Debug for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            MessageType::CreateAccount(account) => write!(f, "CreateAccount({:?})", account),
            MessageType::GetAccount(identifier) => write!(f, "GetAccount({:?})", identifier),
            MessageType::GetAccounts => write!(f, "GetAccounts"),
            MessageType::CallAccountMethod { account_id, method } => write!(
                f,
                "CallAccountMethod{{ account_id: {:?}, method: {:?} }}",
                account_id, method
            ),
            #[cfg(feature = "stronghold")]
            MessageType::ClearStrongholdPassword => write!(f, "ClearStrongholdPassword"),
            #[cfg(feature = "stronghold")]
            MessageType::IsStrongholdPasswordAvailable => write!(f, "IsStrongholdPasswordAvailable"),
            #[cfg(feature = "stronghold")]
            MessageType::Backup {
                destination,
                password: _,
            } => write!(f, "Backup{{ destination: {:?} }}", destination),
            MessageType::RecoverAccounts {
                account_gap_limit,
                address_gap_limit,
            } => write!(
                f,
                "RecoverAccounts{{ account_gap_limit: {:?}, address_gap_limit: {:?} }}",
                account_gap_limit, address_gap_limit
            ),
            #[cfg(feature = "stronghold")]
            MessageType::RestoreBackup { source, password: _ } => write!(f, "RestoreBackup{{ source: {:?} }}", source),
            #[cfg(feature = "storage")]
            MessageType::DeleteStorage => write!(f, "DeleteStorage"),
            MessageType::GenerateMnemonic => write!(f, "GenerateMnemonic"),
            MessageType::VerifyMnemonic(_) => write!(f, "VerifyMnemonic(<omitted>)"),
            MessageType::SetClientOptions(options) => write!(f, "SetClientOptions({:?})", options),
            MessageType::GetNodeInfo { url, auth: _ } => write!(f, "GetNodeInfo{{ url: {:?} }}", url),
            MessageType::SetStrongholdPassword(_) => write!(f, "SetStrongholdPassword(<omitted>)"),
            MessageType::StoreMnemonic(_) => write!(f, "StoreMnemonic(<omitted>)"),
            MessageType::StartBackgroundSync { options, interval } => write!(
                f,
                "StartBackgroundSync{{ options: {:?}, interval: {:?} }}",
                options, interval
            ),
            MessageType::StopBackgroundSync => write!(f, "StopBackgroundSync"),
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            MessageType::EmitTestEvent(event) => write!(f, "EmitTestEvent({:?})", event),
        }
    }
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
                serializer.serialize_unit_variant("MessageType", 6, "RecoverAccounts")
            }
            #[cfg(feature = "stronghold")]
            MessageType::RestoreBackup { .. } => serializer.serialize_unit_variant("MessageType", 7, "RestoreBackup"),
            MessageType::GenerateMnemonic => serializer.serialize_unit_variant("MessageType", 8, "GenerateMnemonic"),
            MessageType::VerifyMnemonic(_) => serializer.serialize_unit_variant("MessageType", 9, "VerifyMnemonic"),
            MessageType::DeleteStorage => serializer.serialize_unit_variant("MessageType", 10, "DeleteStorage"),
            MessageType::SetClientOptions(_) => {
                serializer.serialize_unit_variant("MessageType", 11, "SetClientOptions")
            }
            MessageType::GetNodeInfo { .. } => serializer.serialize_unit_variant("MessageType", 12, "GetNodeInfo"),
            MessageType::SetStrongholdPassword(_) => {
                serializer.serialize_unit_variant("MessageType", 13, "SetStrongholdPassword")
            }
            MessageType::StoreMnemonic(_) => serializer.serialize_unit_variant("MessageType", 14, "StoreMnemonic"),
            MessageType::StartBackgroundSync { .. } => {
                serializer.serialize_unit_variant("MessageType", 15, "StartBackgroundSync")
            }
            MessageType::StopBackgroundSync => {
                serializer.serialize_unit_variant("MessageType", 16, "StopBackgroundSync")
            }
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            MessageType::EmitTestEvent(_) => serializer.serialize_unit_variant("MessageType", 17, "EmitTestEvent"),
            #[cfg(feature = "stronghold")]
            MessageType::ClearStrongholdPassword => {
                serializer.serialize_unit_variant("MessageType", 18, "ClearStrongholdPassword")
            }
            #[cfg(feature = "stronghold")]
            MessageType::IsStrongholdPasswordAvailable => {
                serializer.serialize_unit_variant("MessageType", 19, "IsStrongholdPasswordAvailable")
            }
        }
    }
}

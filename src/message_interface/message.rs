// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    fmt::{Debug, Formatter, Result},
    path::PathBuf,
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
pub enum Message {
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
    /// Change the Stronghold password to another one and also reencrypt the values in the loaded snapshot with it.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "stronghold")]
    ChangeStrongholdPassword {
        password: String,
        #[serde(rename = "keysToReEncrypt")]
        keys_to_re_encrypt: Option<Vec<Vec<u8>>>,
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
    /// Set the stronghold password clear interval.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetStrongholdPasswordClearInterval(Option<u64>),
    /// Store a mnemonic into the Stronghold vault.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    StoreMnemonic(String),
    /// Start background syncing.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    StartBackgroundSync {
        /// Sync options
        options: Option<SyncOptions>,
        /// Interval in milliseconds
        #[serde(rename = "intervalInMilliseconds")]
        interval_in_milliseconds: Option<u64>,
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
impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Message::CreateAccount(account) => write!(f, "CreateAccount({:?})", account),
            Message::GetAccount(identifier) => write!(f, "GetAccount({:?})", identifier),
            Message::GetAccounts => write!(f, "GetAccounts"),
            Message::CallAccountMethod { account_id, method } => write!(
                f,
                "CallAccountMethod{{ account_id: {:?}, method: {:?} }}",
                account_id, method
            ),
            #[cfg(feature = "stronghold")]
            Message::ChangeStrongholdPassword {
                password: _,
                keys_to_re_encrypt,
            } => write!(
                f,
                "ChangeStrongholdPassword{{ password: <omitted>, keys_to_re_encrypt: {:?} }}",
                keys_to_re_encrypt
            ),
            #[cfg(feature = "stronghold")]
            Message::ClearStrongholdPassword => write!(f, "ClearStrongholdPassword"),
            #[cfg(feature = "stronghold")]
            Message::IsStrongholdPasswordAvailable => write!(f, "IsStrongholdPasswordAvailable"),
            #[cfg(feature = "stronghold")]
            Message::Backup {
                destination,
                password: _,
            } => write!(f, "Backup{{ destination: {:?} }}", destination),
            Message::RecoverAccounts {
                account_gap_limit,
                address_gap_limit,
            } => write!(
                f,
                "RecoverAccounts{{ account_gap_limit: {:?}, address_gap_limit: {:?} }}",
                account_gap_limit, address_gap_limit
            ),
            #[cfg(feature = "stronghold")]
            Message::RestoreBackup { source, password: _ } => write!(f, "RestoreBackup{{ source: {:?} }}", source),
            #[cfg(feature = "storage")]
            Message::DeleteStorage => write!(f, "DeleteStorage"),
            Message::GenerateMnemonic => write!(f, "GenerateMnemonic"),
            Message::VerifyMnemonic(_) => write!(f, "VerifyMnemonic(<omitted>)"),
            Message::SetClientOptions(options) => write!(f, "SetClientOptions({:?})", options),
            Message::GetNodeInfo { url, auth: _ } => write!(f, "GetNodeInfo{{ url: {:?} }}", url),
            Message::SetStrongholdPassword(_) => write!(f, "SetStrongholdPassword(<omitted>)"),
            Message::SetStrongholdPasswordClearInterval(interval_in_milliseconds) => {
                write!(f, "SetStrongholdPassword({:?})", interval_in_milliseconds)
            }
            Message::StoreMnemonic(_) => write!(f, "StoreMnemonic(<omitted>)"),
            Message::StartBackgroundSync {
                options,
                interval_in_milliseconds,
            } => write!(
                f,
                "StartBackgroundSync{{ options: {:?}, interval: {:?} }}",
                options, interval_in_milliseconds
            ),
            Message::StopBackgroundSync => write!(f, "StopBackgroundSync"),
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            Message::EmitTestEvent(event) => write!(f, "EmitTestEvent({:?})", event),
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Message::CreateAccount(_) => serializer.serialize_unit_variant("Message", 1, "CreateAccount"),
            Message::GetAccount(_) => serializer.serialize_unit_variant("Message", 2, "GetAccount"),
            Message::GetAccounts => serializer.serialize_unit_variant("Message", 3, "GetAccounts"),
            Message::CallAccountMethod { .. } => serializer.serialize_unit_variant("Message", 4, "CallAccountMethod"),
            #[cfg(feature = "stronghold")]
            Message::Backup { .. } => serializer.serialize_unit_variant("Message", 5, "Backup"),
            Message::RecoverAccounts { .. } => serializer.serialize_unit_variant("Message", 6, "RecoverAccounts"),
            #[cfg(feature = "stronghold")]
            Message::RestoreBackup { .. } => serializer.serialize_unit_variant("Message", 7, "RestoreBackup"),
            Message::GenerateMnemonic => serializer.serialize_unit_variant("Message", 8, "GenerateMnemonic"),
            Message::VerifyMnemonic(_) => serializer.serialize_unit_variant("Message", 9, "VerifyMnemonic"),
            Message::DeleteStorage => serializer.serialize_unit_variant("Message", 10, "DeleteStorage"),
            Message::SetClientOptions(_) => serializer.serialize_unit_variant("Message", 11, "SetClientOptions"),
            Message::GetNodeInfo { .. } => serializer.serialize_unit_variant("Message", 12, "GetNodeInfo"),
            #[cfg(feature = "stronghold")]
            Message::SetStrongholdPassword(_) => {
                serializer.serialize_unit_variant("Message", 13, "SetStrongholdPassword")
            }
            #[cfg(feature = "stronghold")]
            Message::SetStrongholdPasswordClearInterval(_) => {
                serializer.serialize_unit_variant("Message", 14, "SetStrongholdPassword")
            }
            #[cfg(feature = "stronghold")]
            Message::StoreMnemonic(_) => serializer.serialize_unit_variant("Message", 15, "StoreMnemonic"),
            Message::StartBackgroundSync { .. } => {
                serializer.serialize_unit_variant("Message", 16, "StartBackgroundSync")
            }
            Message::StopBackgroundSync => serializer.serialize_unit_variant("Message", 17, "StopBackgroundSync"),
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            Message::EmitTestEvent(_) => serializer.serialize_unit_variant("Message", 18, "EmitTestEvent"),
            #[cfg(feature = "stronghold")]
            Message::ClearStrongholdPassword => {
                serializer.serialize_unit_variant("Message", 19, "ClearStrongholdPassword")
            }
            #[cfg(feature = "stronghold")]
            Message::IsStrongholdPasswordAvailable => {
                serializer.serialize_unit_variant("Message", 20, "IsStrongholdPasswordAvailable")
            }
            #[cfg(feature = "stronghold")]
            Message::ChangeStrongholdPassword { .. } => {
                serializer.serialize_unit_variant("Message", 21, "ChangeStrongholdPassword")
            }
        }
    }
}

// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    account::{Account, AccountIdentifier, SyncedAccount},
    address::Address,
    client::ClientOptions,
    message::{Message as WalletMessage, MessageType as WalletMessageType, Transfer},
    WalletError,
};
use serde::{ser::Serializer, Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

/// An account to create.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct AccountToCreate {
    /// The node options.
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptions,
    /// The account mnemonic.
    pub mnemonic: Option<String>,
    /// The account alias.
    pub alias: Option<String>,
    /// The account createdAt date string.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    GenerateAddress,
    /// List messages.
    ListMessages {
        /// Message type filter.
        #[serde(rename = "messageType")]
        message_type: Option<WalletMessageType>,
        /// Number of messages to get.
        #[serde(default)]
        count: usize,
        /// Number of messages to skip.
        #[serde(default)]
        from: usize,
    },
    /// List addresses.
    ListAddresses {
        /// Address unspent filter.
        #[serde(default)]
        unspent: bool,
    },
    /// Get available balance.
    GetAvailableBalance,
    /// Get total balance.
    GetTotalBalance,
    /// Get latest address.
    GetLatestAddress,
    /// Sync the account.
    SyncAccount {
        /// The first address index to sync.
        #[serde(rename = "addressIndex")]
        address_index: Option<usize>,
        /// The gap limit.
        #[serde(rename = "gapLimit")]
        gap_limit: Option<usize>,
        /// Whether to skip writing the account in storage or not (defaults to false).
        #[serde(rename = "skipPersistance")]
        skip_persistance: Option<bool>,
    },
}

/// The messages that can be sent to the actor.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum MessageType {
    /// Remove the account related to the specified `account_id`.
    RemoveAccount(AccountIdentifier),
    /// Creates an account.
    CreateAccount(AccountToCreate),
    /// Read account.
    GetAccount(AccountIdentifier),
    /// Read accounts.
    GetAccounts,
    /// Consume an account method.
    CallAccountMethod {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The account method to call.
        method: AccountMethod,
    },
    /// Sync accounts.
    SyncAccounts,
    /// Reattach message.
    Reattach {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The message to reattach.
        #[serde(rename = "message_id")]
        message_id: String,
    },
    /// Backup storage.
    Backup(String),
    /// Import accounts from storage.
    #[cfg(any(feature = "stronghold", feature = "sqlite"))]
    RestoreBackup(String),
    /// Set stronghold snapshot password.
    #[cfg(feature = "stronghold")]
    SetStrongholdPassword(String),
    /// Send funds.
    SendTransfer {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The transfer details.
        transfer: Transfer,
    },
    /// Move funds on stored accounts.
    InternalTransfer {
        /// The source account identifier.
        #[serde(rename = "fromAccountId")]
        from_account_id: AccountIdentifier,
        /// The destination account identifier.
        #[serde(rename = "toAccountId")]
        to_account_id: AccountIdentifier,
        /// The transfer amount.
        amount: u64,
    },
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MessageType::RemoveAccount(_) => {
                serializer.serialize_unit_variant("MessageType", 0, "RemoveAccount")
            }
            MessageType::CreateAccount(_) => {
                serializer.serialize_unit_variant("MessageType", 1, "CreateAccount")
            }
            MessageType::GetAccount(_) => {
                serializer.serialize_unit_variant("MessageType", 2, "GetAccount")
            }
            MessageType::GetAccounts => {
                serializer.serialize_unit_variant("MessageType", 3, "GetAccounts")
            }
            MessageType::CallAccountMethod {
                account_id: _,
                method: _,
            } => serializer.serialize_unit_variant("MessageType", 4, "CallAccountMethod"),
            MessageType::SyncAccounts => {
                serializer.serialize_unit_variant("MessageType", 5, "SyncAccounts")
            }
            MessageType::Reattach {
                account_id: _,
                message_id: _,
            } => serializer.serialize_unit_variant("MessageType", 6, "Reattach"),
            MessageType::Backup(_) => serializer.serialize_unit_variant("MessageType", 7, "Backup"),
            #[cfg(any(feature = "stronghold", feature = "sqlite"))]
            MessageType::RestoreBackup(_) => {
                serializer.serialize_unit_variant("MessageType", 8, "RestoreBackup")
            }
            #[cfg(feature = "stronghold")]
            MessageType::SetStrongholdPassword(_) => {
                serializer.serialize_unit_variant("MessageType", 9, "SetStrongholdPassword")
            }
            MessageType::SendTransfer {
                account_id: _,
                transfer: _,
            } => serializer.serialize_unit_variant("MessageType", 10, "SendTransfer"),
            MessageType::InternalTransfer {
                from_account_id: _,
                to_account_id: _,
                amount: _,
            } => serializer.serialize_unit_variant("MessageType", 11, "InternalTransfer"),
        }
    }
}

/// The actor response type.
#[derive(Serialize, Debug)]
pub struct Response {
    id: String,
    #[serde(flatten)]
    response: ResponseType,
    action: MessageType,
}

impl Response {
    /// Creates a new response.
    pub fn new<S: Into<String>>(id: S, action: MessageType, response: ResponseType) -> Self {
        Self {
            id: id.into(),
            response,
            action,
        }
    }

    /// The response's type.
    pub fn response(&self) -> &ResponseType {
        &self.response
    }
}

/// The response message.
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseType {
    /// Account succesfully removed.
    RemovedAccount(AccountIdentifier),
    /// Account succesfully created.
    CreatedAccount(Account),
    /// GetAccount response.
    ReadAccount(Account),
    /// GetAccounts response.
    ReadAccounts(Vec<Account>),
    /// ListMessages response.
    Messages(Vec<WalletMessage>),
    /// ListAddresses response.
    Addresses(Vec<Address>),
    /// GenerateAddress response.
    GeneratedAddress(Address),
    /// GetLatestAddress response.
    LatestAddress(Option<Address>),
    /// GetAvailableBalance response.
    AvailableBalance(u64),
    /// GetTotalBalance response.
    TotalBalance(u64),
    /// SyncAccounts response.
    SyncedAccounts(Vec<SyncedAccount>),
    /// SyncAccount response.
    SyncedAccount(SyncedAccount),
    /// Reattach response.
    Reattached(String),
    /// Backup response.
    BackupSuccessful,
    /// ImportAccounts response.
    BackupRestored,
    /// SetStrongholdPassword response.
    StrongholdPasswordSet,
    /// SendTransfer and InternalTransfer response.
    SentTransfer(WalletMessage),
    /// An error occurred.
    Error(WalletError),
    /// A panic occurred.
    Panic(String),
}

/// The message type.
#[derive(Debug, Clone)]
pub struct Message {
    id: String,
    pub(crate) message_type: MessageType,
    pub(crate) response_tx: UnboundedSender<Response>,
}

impl Message {
    /// Creates a new instance of a Message.
    pub fn new<S: Into<String>>(
        id: S,
        message_type: MessageType,
        response_tx: UnboundedSender<Response>,
    ) -> Self {
        Self {
            id: id.into(),
            message_type,
            response_tx,
        }
    }

    /// The message type.
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    /// The response sender.
    pub fn response_tx(&self) -> &UnboundedSender<Response> {
        &self.response_tx
    }

    /// The message identifier.
    pub fn id(&self) -> &String {
        &self.id
    }
}

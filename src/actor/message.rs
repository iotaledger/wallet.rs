// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        operations::{
            address_generation::AddressGenerationOptions, syncing::SyncOptions,
            transfer::{TransferOptions, TransferOutput, TransferResult},
        },
        types::{
            address::{AccountAddress, AddressWithBalance}, AccountBalance, AccountIdentifier, OutputData, Transaction
        },
        Account,
    },
    client::options::ClientOptions,
    Error,
};

use serde::{ser::Serializer, Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use std::{path::PathBuf, time::Duration};

/// An account to create.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountToCreate {
    /// The account alias.
    pub alias: Option<String>
}

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// List addresses.
    ListAddresses,
    /// Returns only addresses of the account with balance
    ListAddressesWithBalance,
    /// Returns all outputs of the account
    ListOutputs,
    /// Returns all unspent outputs of the account
    ListUnspentOutputs,
    /// Returns all transaction of the account
    ListTransactions,
    /// Returns all pending transaction of the account
    ListPendingTransactions,
    /// Get account balance information.
    GetBalance,
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>
    },
}


/// The messages that can be sent to the actor.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd", content = "payload", rename_all = "camelCase")]
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
        outputs: Vec<TransferOutput>,
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

/// The actor response type.
#[derive(Serialize, Debug)]
pub struct Response {
    #[serde(flatten)]
    response: ResponseType,
    action: MessageType,
}

impl Response {
    /// Creates a new response.
    pub fn new(action: MessageType, response: ResponseType) -> Self {
        Self {
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
    /// Account succesfully created.
    CreatedAccount(Account),
    /// GetAccount response.
    ReadAccount(Account),
    /// GetAccounts response.
    ReadAccounts(Vec<Account>),
    /// ListAddresses
    Addresses(Vec<AccountAddress>),
    /// ListAddressesWithBalance.
    AddressesWithBalance(Vec<AddressWithBalance>),
    /// ListOutputs/ListUnspentOutputs.
    Outputs(Vec<OutputData>),
    /// ListTransactions/ListPendingTransactions.
    Transactions(Vec<Transaction>),
    /// GenerateAddress response.
    GeneratedAddress(Vec<AccountAddress>),
    /// GetBalance response.
    Balance(AccountBalance),
    /// SyncAccount response.
    SyncedAccount(AccountBalance),
    /// Backup response.
    BackupSuccessful,
    /// ImportAccounts response.
    BackupRestored,
    /// SendTransfer and InternalTransfer response.
    SentTransfer(TransferResult),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// GenerateMnemonic response.
    GeneratedMnemonic(String),
    /// VerifyMnemonic response.
    VerifiedMnemonic,
    // /// SetAlias response.
    // UpdatedAlias,
    /// DeleteStorage response.
    DeletedStorage,
    /// SetClientOptions response.
    UpdatedAllClientOptions,
    /// All went fine.
    Ok(()),
}

/// The message type.
#[derive(Debug, Clone)]
pub struct Message {
    pub(crate) message_type: MessageType,
    pub(crate) response_tx: UnboundedSender<Response>,
}

impl Message {
    /// Creates a new instance of a Message.
    pub fn new(message_type: MessageType, response_tx: UnboundedSender<Response>) -> Self {
        Self {
            message_type,
            response_tx,
        }
    }

    /// The message type.
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    /// The message type.
    pub(crate) fn message_type_mut(&mut self) -> &mut MessageType {
        &mut self.message_type
    }

    /// The response sender.
    pub fn response_tx(&self) -> &UnboundedSender<Response> {
        &self.response_tx
    }
}

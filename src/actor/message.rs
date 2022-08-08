// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountBalance, AccountIdentifier, SyncedAccount},
    account_manager::{migration::MigrationAddress, MigratedBundle, MigrationBundle, MigrationData, MinedBundle},
    address::Address,
    client::ClientOptions,
    message::{Message as WalletMessage, MessageType as WalletMessageType, TransferBuilder},
    signing::SignerType,
    Error,
};
use chrono::{DateTime, Local};
use iota_client::NodeInfoWrapper;
use iota_migration::{ternary::T3B1Buf, transaction::bundled::BundledTransactionField};
use serde::{ser::Serializer, Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use std::{num::NonZeroU64, path::PathBuf, time::Duration};

/// An account to create.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountToCreate {
    /// The node options.
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptions,
    /// The account alias.
    pub alias: Option<String>,
    /// The account createdAt date string.
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Local>>,
    /// Whether to skip saving the account to storage or not.
    #[serde(rename = "skipPersistence", default)]
    pub skip_persistence: bool,
    /// The account's signer type.
    #[serde(rename = "signerType")]
    pub signer_type: Option<SignerType>,
    /// Allow to create an account with multiple empty accounts
    #[serde(rename = "allowCreateMultipleEmptyAccounts", default)]
    pub allow_create_multiple_empty_accounts: bool,
}

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    GenerateAddress,
    /// Get a unused address.
    GetUnusedAddress,
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
    ListAddresses,
    /// List spent addresses.
    ListSpentAddresses,
    /// List unspent addresses.
    ListUnspentAddresses,
    /// Get account balance information.
    GetBalance,
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
        #[serde(rename = "skipPersistence")]
        skip_persistence: Option<bool>,
    },
    /// Checks if the account's latest address is unused after syncing with the Tangle.
    IsLatestAddressUnused,
    /// Updates the account alias.
    SetAlias(String),
    /// Updates the account client options.
    SetClientOptions(Box<ClientOptions>),
    /// Gets the node information.
    GetNodeInfo(Option<String>, Option<String>, Option<(String, String)>),
}

/// The returned account.
#[derive(Debug, Serialize)]
pub struct AccountDto {
    /// Inner account object.
    #[serde(flatten)]
    pub account: Account,
    /// Message history.
    pub messages: Vec<WalletMessage>,
}

impl AccountDto {
    /// Creates a new instance of the account DTO.
    pub fn new(account: Account, messages: Vec<WalletMessage>) -> Self {
        Self { account, messages }
    }
}

/// The messages that can be sent to the actor.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum MessageType {
    /// Remove the account related to the specified `account_id`.
    RemoveAccount(AccountIdentifier),
    /// Creates an account.
    CreateAccount(Box<AccountToCreate>),
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
    SyncAccounts {
        /// The first address index to sync.
        #[serde(rename = "addressIndex")]
        address_index: Option<usize>,
        /// The gap limit.
        #[serde(rename = "gapLimit")]
        gap_limit: Option<usize>,
        /// Minimum number of accounts to check on discovery.
        #[serde(rename = "accountDiscoveryThreshold")]
        account_discovery_threshold: Option<usize>,
    },
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
    Backup {
        /// The backup destination.
        destination: PathBuf,
        /// Stronghold file password.
        password: String,
    },
    /// Import accounts from storage.
    RestoreBackup {
        /// The path to the backed up storage.
        #[serde(rename = "backupPath")]
        backup_path: String,
        /// Stronghold file password.
        password: String,
    },
    /// Sets the password used to encrypt/decrypt the storage.
    SetStoragePassword(String),
    /// Clears the password used to encrypt/decrypt the storage.
    ClearStoragePassword,
    /// Set stronghold snapshot password.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    SetStrongholdPassword(String),
    /// Sets the password clear interval.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    SetStrongholdPasswordClearInterval(Duration),
    /// Get stronghold status.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    GetStrongholdStatus,
    /// Lock the stronghold snapshot (clears password and unload snapshot from memory).
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    LockStronghold,
    /// Send funds.
    SendTransfer {
        /// The account identifier.
        #[serde(rename = "accountId")]
        account_id: AccountIdentifier,
        /// The transfer details.
        transfer: Box<TransferBuilder>,
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
        amount: NonZeroU64,
    },
    /// Generates a new mnemonic.
    GenerateMnemonic,
    /// Checks if the given mnemonic is valid.
    VerifyMnemonic(String),
    /// Store mnemonic.
    StoreMnemonic {
        /// The signer type.
        #[serde(rename = "signerType")]
        signer_type: SignerType,
        /// The mnemonic. If empty, we'll generate one.
        mnemonic: Option<String>,
    },
    /// Checks if all accounts has unused latest address after syncing with the Tangle.
    IsLatestAddressUnused,
    /// Get the Ledger Nano or Speculos simulator status.
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
    GetLedgerStatus(bool),
    /// Deletes the storage.
    DeleteStorage,
    /// Changes stronghold snapshot password.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    ChangeStrongholdPassword {
        /// The current stronghold password.
        #[serde(rename = "currentPassword")]
        current_password: String,
        /// The new stronghold password.
        #[serde(rename = "newPassword")]
        new_password: String,
    },
    /// Updates the client options for all accounts.
    SetClientOptions(Box<ClientOptions>),
    /// Get legacy network balance for the seed.
    GetMigrationData {
        /// The nodes to connect to.
        nodes: Vec<String>,
        /// The permanode to use.
        permanode: Option<String>,
        /// The legacy seed.
        seed: String,
        /// The WOTS address security level.
        #[serde(rename = "securityLevel")]
        security_level: Option<u8>,
        /// The gap limit for how many addresses are generated in one round
        #[serde(rename = "gapLimit")]
        gap_limit: Option<u64>,
        /// The initial address index.
        #[serde(rename = "initialAddressIndex")]
        initial_address_index: Option<u64>,
    },
    /// Get legacy network balance for addresses.
    GetLedgerMigrationData {
        /// The nodes to connect to.
        nodes: Vec<String>,
        /// The permanode to use.
        permanode: Option<String>,
        /// Address objects as String converted with address and index
        addresses: Vec<String>,
        /// The WOTS address security level.
        #[serde(rename = "securityLevel")]
        security_level: Option<u8>,
    },
    /// Creates the bundle for migration, performs bundle mining if the address was spent and signs the bundle.
    CreateMigrationBundle {
        /// The legacy seed.
        seed: String,
        /// The bundle input address indexes.
        #[serde(rename = "inputAddressIndexes")]
        input_address_indexes: Vec<u64>,
        /// Whether we should perform bundle mining or not.
        mine: bool,
        /// Timeout in seconds for the bundle mining process.
        #[serde(rename = "timeoutSeconds")]
        timeout_secs: u64,
        /// Offset for the bundle mining process.
        offset: i64,
        /// The name of the log file (stored on the storage folder).
        #[serde(rename = "logFileName")]
        log_file_name: String,
    },
    /// Sends the migration bundle associated with the hash.
    SendMigrationBundle {
        /// Node URLs.
        nodes: Vec<String>,
        /// Bundle hash returned on `CreateMigrationBundle`.
        #[serde(rename = "bundleHash")]
        bundle_hash: String,
        /// Minimum weight magnitude.
        mwm: u8,
    },
    /// Sends the migration bundle associated with the hash.
    SendLedgerMigrationBundle {
        /// Node URLs.
        nodes: Vec<String>,
        /// Bundle tx trytes.
        bundle: Vec<String>,
        /// Minimum weight magnitude.
        mwm: u8,
    },
    /// Get seed checksum.
    GetSeedChecksum(String),
    /// Get migration address
    GetMigrationAddress {
        /// Display prompt on the ledger
        ledger_prompt: bool,
        /// Account identifier
        account_id: AccountIdentifier,
    },
    /// Mine bundle
    MineBundle {
        /// Prepared bundle.
        #[serde(rename = "preparedBundle")]
        prepared_bundle: Vec<String>,
        /// Known spent bundle hashes.
        #[serde(rename = "spentBundleHashes")]
        spent_bundle_hashes: Vec<String>,
        /// Security level.
        #[serde(rename = "securityLevel")]
        security_level: u8,
        /// Mining timeout in seconds.
        timeout: u64,
        /// Mining offset.
        offset: i64,
    },
    /// Get the checksum for a legacy address.
    GetLegacyAddressChecksum(String),
    /// Start background syncing.
    StartBackgroundSync {
        /// Polling interval, time between each syncing operation
        #[serde(rename = "pollingInterval")]
        polling_interval: Duration,
        /// If outputs should get automatically consolidated
        #[serde(rename = "automaticOutputConsolidation")]
        automatic_output_consolidation: bool,
        /// gap_limit for the unused address generation
        #[serde(rename = "gapLimit")]
        gap_limit: Option<usize>,
    },
    /// Stop background syncing.
    StopBackgroundSync,
    #[cfg(feature = "participation")]
    /// Participate in a voting or staking events
    Participate {
        /// account which should participate
        account_identifier: AccountIdentifier,
        /// participation information to send
        participations: Vec<crate::participation::types::Participation>,
    },
    #[cfg(feature = "participation")]
    /// Stop participating in a voting or staking events
    StopParticipating {
        /// account which should stop participate
        account_identifier: AccountIdentifier,
        /// Event ids to stop participating from
        event_ids: Vec<String>,
    },
    #[cfg(feature = "participation")]
    /// Get participating overview
    GetParticipationOverview {
        #[serde(rename = "assemblyEventId")]
        /// Event id for assembly
        assembly_event_id: String,
    },
    #[cfg(feature = "participation")]
    /// Get participation events
    GetParticipationEvents,
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MessageType::RemoveAccount(_) => serializer.serialize_unit_variant("MessageType", 0, "RemoveAccount"),
            MessageType::CreateAccount(_) => serializer.serialize_unit_variant("MessageType", 1, "CreateAccount"),
            MessageType::GetAccount(_) => serializer.serialize_unit_variant("MessageType", 2, "GetAccount"),
            MessageType::GetAccounts => serializer.serialize_unit_variant("MessageType", 3, "GetAccounts"),
            MessageType::CallAccountMethod { .. } => {
                serializer.serialize_unit_variant("MessageType", 4, "CallAccountMethod")
            }
            MessageType::SyncAccounts { .. } => serializer.serialize_unit_variant("MessageType", 5, "SyncAccounts"),
            MessageType::Reattach { .. } => serializer.serialize_unit_variant("MessageType", 6, "Reattach"),
            MessageType::Backup { .. } => serializer.serialize_unit_variant("MessageType", 7, "Backup"),
            MessageType::RestoreBackup { .. } => serializer.serialize_unit_variant("MessageType", 8, "RestoreBackup"),
            MessageType::SetStoragePassword(_) => {
                serializer.serialize_unit_variant("MessageType", 9, "SetStoragePassword")
            }
            MessageType::ClearStoragePassword => {
                serializer.serialize_unit_variant("MessageType", 10, "ClearStoragePassword")
            }
            #[cfg(feature = "stronghold")]
            MessageType::SetStrongholdPassword(_) => {
                serializer.serialize_unit_variant("MessageType", 11, "SetStrongholdPassword")
            }
            #[cfg(feature = "stronghold")]
            MessageType::SetStrongholdPasswordClearInterval(_) => {
                serializer.serialize_unit_variant("MessageType", 12, "SetStrongholdPasswordClearInterval")
            }
            #[cfg(feature = "stronghold")]
            MessageType::GetStrongholdStatus => {
                serializer.serialize_unit_variant("MessageType", 13, "GetStrongholdStatus")
            }
            #[cfg(feature = "stronghold")]
            MessageType::LockStronghold => serializer.serialize_unit_variant("MessageType", 14, "LockStronghold"),
            MessageType::SendTransfer { .. } => serializer.serialize_unit_variant("MessageType", 15, "SendTransfer"),
            MessageType::InternalTransfer { .. } => {
                serializer.serialize_unit_variant("MessageType", 16, "InternalTransfer")
            }
            MessageType::GenerateMnemonic => serializer.serialize_unit_variant("MessageType", 17, "GenerateMnemonic"),
            MessageType::VerifyMnemonic(_) => serializer.serialize_unit_variant("MessageType", 18, "VerifyMnemonic"),
            MessageType::StoreMnemonic { .. } => serializer.serialize_unit_variant("MessageType", 19, "StoreMnemonic"),
            MessageType::IsLatestAddressUnused => {
                serializer.serialize_unit_variant("MessageType", 20, "IsLatestAddressUnused")
            }
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            MessageType::GetLedgerStatus(_) => serializer.serialize_unit_variant("MessageType", 21, "GetLedgerStatus"),
            MessageType::DeleteStorage => serializer.serialize_unit_variant("MessageType", 22, "DeleteStorage"),
            #[cfg(feature = "stronghold")]
            MessageType::ChangeStrongholdPassword { .. } => {
                serializer.serialize_unit_variant("MessageType", 23, "ChangeStrongholdPassword")
            }
            MessageType::SetClientOptions(_) => {
                serializer.serialize_unit_variant("MessageType", 24, "SetClientOptions")
            }
            MessageType::GetMigrationData { .. } => {
                serializer.serialize_unit_variant("MessageType", 25, "GetMigrationData")
            }
            MessageType::CreateMigrationBundle { .. } => {
                serializer.serialize_unit_variant("MessageType", 26, "CreateMigrationBundle")
            }
            MessageType::SendMigrationBundle { .. } => {
                serializer.serialize_unit_variant("MessageType", 27, "SendMigrationBundle")
            }
            MessageType::GetSeedChecksum(_) => serializer.serialize_unit_variant("MessageType", 28, "GetSeedChecksum"),
            MessageType::GetMigrationAddress { .. } => {
                serializer.serialize_unit_variant("MessageType", 29, "GetMigrationAddress")
            }
            MessageType::MineBundle { .. } => serializer.serialize_unit_variant("MessageType", 30, "MineBundle"),
            MessageType::GetLedgerMigrationData { .. } => {
                serializer.serialize_unit_variant("MessageType", 31, "GetLedgerMigrationData")
            }
            MessageType::SendLedgerMigrationBundle { .. } => {
                serializer.serialize_unit_variant("MessageType", 32, "SendLedgerMigrationBundle")
            }
            MessageType::GetLegacyAddressChecksum(_) => {
                serializer.serialize_unit_variant("MessageType", 33, "GetLegacyAddressChecksum")
            }
            MessageType::StartBackgroundSync { .. } => {
                serializer.serialize_unit_variant("MessageType", 34, "StartBackgroundSync")
            }
            MessageType::StopBackgroundSync => {
                serializer.serialize_unit_variant("MessageType", 35, "StopBackgroundSync")
            }
            #[cfg(feature = "participation")]
            MessageType::Participate { .. } => serializer.serialize_unit_variant("MessageType", 36, "Participate"),
            #[cfg(feature = "participation")]
            MessageType::StopParticipating { .. } => {
                serializer.serialize_unit_variant("MessageType", 37, "StopParticipating")
            }
            #[cfg(feature = "participation")]
            MessageType::GetParticipationOverview { .. } => {
                serializer.serialize_unit_variant("MessageType", 38, "GetParticipationOverview")
            }
            #[cfg(feature = "participation")]
            MessageType::GetParticipationEvents => {
                serializer.serialize_unit_variant("MessageType", 39, "GetParticipationEvents")
            }
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

/// Spent address data.
#[derive(Debug, Serialize)]
pub struct MigrationInputDto {
    /// Input address.
    address: String,
    /// Security level.
    #[serde(rename = "securityLevel")]
    security_level: u8,
    /// Balance of the address.
    balance: u64,
    /// Index of the address.
    index: u64,
    /// Spent status.
    spent: bool,
    #[serde(rename = "spentBundleHashes")]
    spent_bundle_hashes: Option<Vec<String>>,
}

/// Legacy information fetched.
#[derive(Debug, Serialize)]
pub struct MigrationDataDto {
    balance: u64,
    #[serde(rename = "lastCheckedAddressIndex")]
    last_checked_address_index: u64,
    inputs: Vec<MigrationInputDto>,
    #[serde(rename = "spentAddresses")]
    spent_addresses: bool,
}

impl From<MigrationData> for MigrationDataDto {
    fn from(data: MigrationData) -> Self {
        let mut inputs: Vec<MigrationInputDto> = Vec::new();
        for input in data.inputs {
            let address = input
                .address
                .to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>();
            inputs.push(MigrationInputDto {
                address,
                security_level: input.security_lvl,
                balance: input.balance,
                index: input.index,
                spent: input.spent,
                spent_bundle_hashes: input.spent_bundlehashes,
            });
        }
        Self {
            balance: data.balance,
            last_checked_address_index: data.last_checked_address_index,
            inputs,
            spent_addresses: data.spent_addresses,
        }
    }
}

/// The response message.
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseType {
    /// Account succesfully removed.
    RemovedAccount(AccountIdentifier),
    /// Account succesfully created.
    CreatedAccount(AccountDto),
    /// GetAccount response.
    ReadAccount(AccountDto),
    /// GetAccounts response.
    ReadAccounts(Vec<AccountDto>),
    /// ListMessages response.
    Messages(Vec<WalletMessage>),
    /// ListAddresses/ListSpentAddresses/ListUnspentAddresses response.
    Addresses(Vec<Address>),
    /// GenerateAddress response.
    GeneratedAddress(Address),
    /// GetUnusedAddress response.
    UnusedAddress(Address),
    /// GetLatestAddress response.
    LatestAddress(Address),
    /// GetBalance response.
    Balance(AccountBalance),
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
    /// SetStoragePassword response.
    StoragePasswordSet,
    /// ClearStoragePassword response.
    StoragePasswordCleared,
    /// SetStrongholdPassword response.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    StrongholdPasswordSet,
    /// SetStrongholdPasswordClearInterval response.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    StrongholdPasswordClearIntervalSet,
    /// GetStrongholdStatus response.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    StrongholdStatus(crate::stronghold::Status),
    /// LockStronghold response.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    LockedStronghold,
    /// SendTransfer and InternalTransfer response.
    SentTransfer(WalletMessage),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// GenerateMnemonic response.
    GeneratedMnemonic(String),
    /// VerifyMnemonic response.
    VerifiedMnemonic,
    /// StoreMnemonic response.
    StoredMnemonic,
    /// AccountMethod's IsLatestAddressUnused response.
    IsLatestAddressUnused(bool),
    /// IsLatestAddressUnused response.
    AreAllLatestAddressesUnused(bool),
    /// SetAlias response.
    UpdatedAlias,
    /// Account method SetClientOptions response.
    UpdatedClientOptions,
    /// GetLedgerStatus response.
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
    LedgerStatus(crate::LedgerStatus),
    /// DeleteStorage response.
    DeletedStorage,
    /// ChangeStrongholdPassword response.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    StrongholdPasswordChanged,
    /// SetClientOptions response.
    UpdatedAllClientOptions,
    /// GetNodeInfo response.
    NodeInfo(NodeInfoWrapper),
    /// GetMigrationData response.
    MigrationData(MigrationDataDto),
    /// CreateMigrationBundle response (bundle hash).
    CreatedMigrationBundle(MigrationBundle),
    /// SendMigrationBundle response.
    SentMigrationBundle(MigratedBundle),
    /// GetSeedChecksum response.
    SeedChecksum(String),
    /// GetMigrationAddress response.
    MigrationAddress(MigrationAddress),
    /// GetMigrationAddress response.
    MineBundle(MinedBundle),
    /// GetLegacyAddressChecksum response.
    GetLegacyAddressChecksum(String),
    /// All went fine.
    Ok(()),
    #[cfg(feature = "participation")]
    /// Sent participations response.
    SentParticipation(Vec<WalletMessage>),
    #[cfg(feature = "participation")]
    /// Participating accounts with their staking rewards.
    ParticipationOverview(crate::participation::types::ParticipatingAccounts),
    #[cfg(feature = "participation")]
    /// Get data about participation events.
    EventsData(Vec<crate::participation::types::EventData>),
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
    pub fn new<S: Into<String>>(id: S, message_type: MessageType, response_tx: UnboundedSender<Response>) -> Self {
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

    /// The message type.
    pub(crate) fn message_type_mut(&mut self) -> &mut MessageType {
        &mut self.message_type
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

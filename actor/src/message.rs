use iota_wallet::{
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
  #[serde(rename = "clientOptions")]
  pub client_options: ClientOptions,
  pub mnemonic: Option<String>,
  pub alias: Option<String>,
  #[serde(rename = "createdAt")]
  pub created_at: Option<String>,
}

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
  GenerateAddress,
  ListMessages {
    #[serde(rename = "messageType")]
    message_type: Option<WalletMessageType>,
    #[serde(default)]
    count: usize,
    #[serde(default)]
    from: usize,
  },
  ListAddresses {
    #[serde(default)]
    unspent: bool,
  },
  GetAvailableBalance,
  GetTotalBalance,
  GetLatestAddress,
  SyncAccount {
    #[serde(rename = "addressIndex")]
    address_index: Option<usize>,
    #[serde(rename = "gapLimit")]
    gap_limit: Option<usize>,
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
  CallAccountMethod {
    #[serde(rename = "accountId")]
    account_id: AccountIdentifier,
    method: AccountMethod,
  },
  /// Sync accounts.
  SyncAccounts,
  /// Reattach message.
  Reattach {
    #[serde(rename = "accountId")]
    account_id: AccountIdentifier,
    #[serde(rename = "message_id")]
    message_id: String,
  },
  Backup(String),
  RestoreBackup(String),
  SetStrongholdPassword(String),
  SendTransfer {
    #[serde(rename = "accountId")]
    account_id: AccountIdentifier,
    transfer: Transfer,
  },
  InternalTransfer {
    #[serde(rename = "fromAccountId")]
    from_account_id: AccountIdentifier,
    #[serde(rename = "toAccountId")]
    to_account_id: AccountIdentifier,
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
      MessageType::RestoreBackup(_) => {
        serializer.serialize_unit_variant("MessageType", 8, "RestoreBackup")
      }
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

#[derive(Serialize, Debug)]
pub struct Response {
  id: String,
  #[serde(flatten)]
  response: ResponseType,
  action: MessageType,
}

impl Response {
  pub fn new<S: Into<String>>(id: S, action: MessageType, response: ResponseType) -> Self {
    Self {
      id: id.into(),
      response,
      action,
    }
  }

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
  ReadAccount(Account),
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
  SyncedAccounts(Vec<SyncedAccount>),
  SyncedAccount(SyncedAccount),
  Reattached(String),
  BackupSuccessful,
  BackupRestored,
  StrongholdPasswordSet,
  SentTransfer(WalletMessage),
  /// An error occurred.
  Error(WalletError),
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

  pub fn message_type(&self) -> &MessageType {
    &self.message_type
  }

  pub fn response_tx(&self) -> &UnboundedSender<Response> {
    &self.response_tx
  }

  pub fn id(&self) -> &String {
    &self.id
  }
}

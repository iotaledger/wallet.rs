use iota_wallet::client::ClientOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

/// An account to create.
#[derive(Debug, Deserialize)]
pub struct AccountToCreate {
  /// The account id.
  id: String,
  client_options: ClientOptions,
}

impl AccountToCreate {
  pub fn new(id: String) -> Self {
    Self { id }
  }

  pub fn id(&self) -> &String {
    &self.id
  }

  pub fn client_options(&self) -> &ClientOptions {
    &self.client_options
  }
}

/// The messages that can be sent to the actor.
#[derive(Debug, Deserialize)]
pub enum MessageType {
  /// Remove the account related to the specified `account_id`.
  RemoveAccount(String),
  /// Creates an account.
  CreateAccount(AccountToCreate),
}

/// The response message.
#[derive(Debug, Serialize, PartialEq)]
pub enum ResponseMessage {
  /// Account succesfully removed.
  RemovedAccount,
  /// Account succesfully created.
  CreatedAccount,
  /// An error occurred.
  Error(String),
}

/// The message type.
#[derive(Debug)]
pub struct Message {
  message_type: MessageType,
  response_tx: UnboundedSender<ResponseMessage>,
}

impl Message {
  /// Creates a new instance of a Message.
  pub fn new(message_type: MessageType, response_tx: UnboundedSender<ResponseMessage>) -> Self {
    Self {
      message_type,
      response_tx,
    }
  }

  pub fn message_type(&self) -> &MessageType {
    &self.message_type
  }

  pub fn response_tx(&self) -> &UnboundedSender<ResponseMessage> {
    &self.response_tx
  }
}

//! The IOTA Wallet Library

#![warn(rust_2018_idioms)]

use chronicle_common::actor;
use tokio::sync::mpsc::UnboundedReceiver;

use iota_wallet::{account_manager::AccountManager, transaction::TransactionType, Result};

/// The message module contains the actor's message and response types.
pub mod message;
use message::*;

actor!(AccountBuilder {
  rx: UnboundedReceiver<Message>,
  account_manager: AccountManager
});

impl AccountBuilder {
  /// Builds the Account actor.
  pub fn build(self) -> Account {
    Account {
      rx: self.rx.expect("rx is required"),
      account_manager: AccountManager::new(),
    }
  }
}

/// The Account actor.
pub struct Account {
  rx: UnboundedReceiver<Message>,
  account_manager: AccountManager,
}

impl Account {
  /// Runs the actor.
  pub async fn run(mut self) {
    println!("running account actor");

    while let Some(message) = self.rx.recv().await {
      let response: Result<ResponseMessage> = match message.message_type() {
        MessageType::RemoveAccount(account_id) => {
          if account_id == "" {
            Ok(ResponseMessage::Error(
              "account_id must be valid".to_string(),
            ))
          } else {
            self.remove_account(account_id)
          }
        }
        MessageType::CreateAccount(account) => self.create_account(account),
        MessageType::ListTransactions {
          account_id,
          count,
          from,
          transaction_type,
        } => self.list_transactions(account_id, count, from, transaction_type),
      };

      let response = match response {
        Ok(r) => r,
        Err(e) => ResponseMessage::Error(e.to_string()),
      };
      message.response_tx().send(response).unwrap();
    }
  }

  fn list_transactions(
    &self,
    account_id: &String,
    count: &u64,
    from: &u64,
    transaction_type: &Option<TransactionType>,
  ) -> Result<ResponseMessage> {
    let transactions = self
      .account_manager
      .get_account(account_id.to_string().into())?
      .list_transactions(*count, *from, transaction_type.clone())
      .into_iter()
      .cloned()
      .collect();
    Ok(ResponseMessage::Transactions(transactions))
  }

  /// The remove account message handler.
  fn remove_account(&self, account_id: &str) -> Result<ResponseMessage> {
    self
      .account_manager
      .remove_account(account_id.to_string().into())
      .map(|_| ResponseMessage::RemovedAccount)
  }

  /// The create account message handler.
  fn create_account(&self, account: &AccountToCreate) -> Result<ResponseMessage> {
    self
      .account_manager
      .create_account(account.client_options())
      .id(account.id())
      .mnemonic(account.id())
      .initialise()
      .map(|_| ResponseMessage::CreatedAccount)
  }
}

#[cfg(test)]
mod tests {
  use super::{
    message::{AccountToCreate, Message, MessageType, ResponseMessage},
    AccountBuilder,
  };
  use iota_wallet::client::ClientOptionsBuilder;
  use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

  fn spawn_actor() -> UnboundedSender<Message> {
    let (tx, rx) = unbounded_channel();
    let actor = AccountBuilder::new().rx(rx).build();
    tokio::spawn(actor.run());
    tx
  }

  async fn send_message(
    tx: &UnboundedSender<Message>,
    message_type: MessageType,
  ) -> ResponseMessage {
    let (message_tx, mut message_rx) = unbounded_channel();
    let message = Message::new(message_type, message_tx);
    tx.send(message).unwrap();
    message_rx.recv().await.unwrap()
  }

  #[tokio::test]
  async fn create_and_remove_account() {
    let tx = spawn_actor();

    let account_id = "some id".to_string();
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
      .expect("invalid node URL")
      .build();

    // create an account
    let account = AccountToCreate::new(account_id.clone(), client_options);
    let response = send_message(&tx, MessageType::CreateAccount(account)).await;
    assert_eq!(response, ResponseMessage::CreatedAccount);

    // remove the created account
    let response = send_message(&tx, MessageType::RemoveAccount(account_id)).await;
    assert_eq!(response, ResponseMessage::RemovedAccount);
  }
}

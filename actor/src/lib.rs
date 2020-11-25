//! The IOTA Wallet Actor

#![warn(rust_2018_idioms)]

use chronicle_common::actor;
use futures::{Future, FutureExt};
use iota::message::prelude::MessageId;
use iota_wallet::{
  account::AccountIdentifier,
  account_manager::AccountManager,
  message::{Message as WalletMessage, Transfer},
  DateTime, Result, Utc,
};
use std::any::Any;
use std::convert::TryInto;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

mod message;
pub use message::*;

pub use iota_wallet as wallet;

actor!(WalletBuilder {
  rx: UnboundedReceiver<Message>,
  message_handler: WalletMessageHandler
});

impl WalletBuilder {
  /// Builds the Wallet actor.
  pub fn build(self) -> Wallet {
    Wallet {
      rx: self.rx.expect("rx is required"),
      message_handler: WalletMessageHandler::new().expect("failed to initialise account manager"),
    }
  }
}

pub struct WalletMessageHandler {
  account_manager: AccountManager,
}

impl Default for WalletMessageHandler {
  fn default() -> Self {
    Self {
      account_manager: AccountManager::new().unwrap(),
    }
  }
}

fn panic_to_response_message(panic: Box<dyn Any>) -> Result<ResponseType> {
  let msg = if let Some(message) = panic.downcast_ref::<String>() {
    format!("Internal error: {}", message)
  } else if let Some(message) = panic.downcast_ref::<&str>() {
    format!("Internal error: {}", message)
  } else {
    "Internal error".to_string()
  };
  let current_backtrace = backtrace::Backtrace::new();
  Ok(ResponseType::Panic(format!(
    "{}\n\n{:?}",
    msg, current_backtrace
  )))
}

fn convert_panics<F: FnOnce() -> Result<ResponseType>>(f: F) -> Result<ResponseType> {
  match catch_unwind(AssertUnwindSafe(|| f())) {
    Ok(result) => result,
    Err(panic) => panic_to_response_message(panic),
  }
}

pub async fn convert_async_panics<F>(f: impl FnOnce() -> F) -> Result<ResponseType>
where
  F: Future<Output = Result<ResponseType>>,
{
  match AssertUnwindSafe(f()).catch_unwind().await {
    Ok(result) => result,
    Err(panic) => panic_to_response_message(panic),
  }
}

impl WalletMessageHandler {
  pub fn new() -> Result<Self> {
    let instance = Self {
      account_manager: AccountManager::new()?,
    };
    Ok(instance)
  }

  pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
    let instance = Self {
      account_manager: AccountManager::with_storage_path(storage_path)?,
    };
    Ok(instance)
  }

  /// Gets the account manager instance.
  pub fn set_polling_interval(&mut self, interval: Duration) {
    self.account_manager.set_polling_interval(interval);
  }

  pub async fn handle(&mut self, message: Message) {
    let response: Result<ResponseType> = match message.message_type() {
      MessageType::RemoveAccount(account_id) => convert_panics(|| self.remove_account(account_id)),
      MessageType::CreateAccount(account) => convert_panics(|| self.create_account(account)),
      MessageType::GetAccount(account_id) => convert_panics(|| self.get_account(account_id)),
      MessageType::GetAccounts => convert_panics(|| self.get_accounts()),
      MessageType::CallAccountMethod { account_id, method } => {
        convert_async_panics(|| async { self.call_account_method(account_id, method).await }).await
      }
      MessageType::SyncAccounts => {
        convert_async_panics(|| async { self.sync_accounts().await }).await
      }
      MessageType::Reattach {
        account_id,
        message_id,
      } => convert_async_panics(|| async { self.reattach(account_id, message_id).await }).await,
      MessageType::Backup(destination_path) => convert_panics(|| self.backup(destination_path)),
      MessageType::RestoreBackup(backup_path) => {
        convert_panics(|| self.restore_backup(backup_path))
      }
      MessageType::SetStrongholdPassword(password) => {
        convert_panics(|| self.set_stronghold_password(password))
      }
      MessageType::SendTransfer {
        account_id,
        transfer,
      } => convert_async_panics(|| async { self.send_transfer(account_id, transfer).await }).await,
      MessageType::InternalTransfer {
        from_account_id,
        to_account_id,
        amount,
      } => {
        convert_async_panics(|| async {
          self
            .internal_transfer(from_account_id, to_account_id, *amount)
            .await
        })
        .await
      }
    };

    let response = match response {
      Ok(r) => r,
      Err(e) => ResponseType::Error(e),
    };
    let _ = message.response_tx.send(Response::new(
      message.id().to_string(),
      message.message_type,
      response,
    ));
  }

  fn backup(&self, destination_path: &str) -> Result<ResponseType> {
    self.account_manager.backup(destination_path)?;
    Ok(ResponseType::BackupSuccessful)
  }

  fn restore_backup(&self, backup_path: &str) -> Result<ResponseType> {
    self.account_manager.import_accounts(backup_path)?;
    Ok(ResponseType::BackupRestored)
  }

  async fn reattach(
    &self,
    account_id: &AccountIdentifier,
    message_id: &str,
  ) -> Result<ResponseType> {
    let parsed_message_id = MessageId::new(
      message_id.as_bytes()[..]
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid message id length"))?,
    );
    self
      .account_manager
      .reattach(*account_id, &parsed_message_id)
      .await?;
    Ok(ResponseType::Reattached(message_id.to_string()))
  }

  async fn sync_accounts(&self) -> Result<ResponseType> {
    let synced = self.account_manager.sync_accounts().await?;
    Ok(ResponseType::SyncedAccounts(synced))
  }

  async fn call_account_method(
    &self,
    account_id: &AccountIdentifier,
    method: &AccountMethod,
  ) -> Result<ResponseType> {
    let mut account = self.account_manager.get_account(*account_id)?;
    match method {
      AccountMethod::GenerateAddress => {
        let address = account.generate_address().await?;
        Ok(ResponseType::GeneratedAddress(address))
      }
      AccountMethod::ListMessages {
        count,
        from,
        message_type,
      } => {
        let messages: Vec<WalletMessage> = account
          .list_messages(*count, *from, message_type.clone())
          .into_iter()
          .cloned()
          .collect();
        Ok(ResponseType::Messages(messages))
      }
      AccountMethod::ListAddresses { unspent } => {
        let addresses = account
          .list_addresses(*unspent)
          .into_iter()
          .cloned()
          .collect();
        Ok(ResponseType::Addresses(addresses))
      }
      AccountMethod::GetAvailableBalance => {
        Ok(ResponseType::AvailableBalance(account.available_balance()))
      }
      AccountMethod::GetTotalBalance => Ok(ResponseType::TotalBalance(account.total_balance())),
      AccountMethod::GetLatestAddress => Ok(ResponseType::LatestAddress(
        account.latest_address().cloned(),
      )),
      AccountMethod::SyncAccount {
        address_index,
        gap_limit,
        skip_persistance,
      } => {
        let mut synchronizer = account.sync();
        if let Some(address_index) = address_index {
          synchronizer = synchronizer.address_index(*address_index);
        }
        if let Some(gap_limit) = gap_limit {
          synchronizer = synchronizer.gap_limit(*gap_limit);
        }
        if let Some(skip_persistance) = skip_persistance {
          if *skip_persistance {
            synchronizer = synchronizer.skip_persistance();
          }
        }
        let synced = synchronizer.execute().await?;
        Ok(ResponseType::SyncedAccount(synced))
      }
    }
  }

  /// The remove account message handler.
  fn remove_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
    self
      .account_manager
      .remove_account(*account_id)
      .map(|_| ResponseType::RemovedAccount(*account_id))
  }

  /// The create account message handler.
  fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
    let mut builder = self
      .account_manager
      .create_account(account.client_options.clone());

    if let Some(mnemonic) = &account.mnemonic {
      builder = builder.mnemonic(mnemonic);
    }
    if let Some(alias) = &account.alias {
      builder = builder.alias(alias);
    }
    if let Some(created_at) = &account.created_at {
      builder = builder.created_at(
        created_at
          .parse::<DateTime<Utc>>()
          .map_err(|e| anyhow::anyhow!(e.to_string()))?,
      );
    }

    builder.initialise().map(ResponseType::CreatedAccount)
  }

  fn get_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
    let account = self.account_manager.get_account(*account_id)?;
    Ok(ResponseType::ReadAccount(account))
  }

  fn get_accounts(&self) -> Result<ResponseType> {
    let accounts = self.account_manager.get_accounts()?;
    Ok(ResponseType::ReadAccounts(accounts))
  }

  fn set_stronghold_password(&mut self, password: &str) -> Result<ResponseType> {
    self.account_manager.set_stronghold_password(password)?;
    Ok(ResponseType::StrongholdPasswordSet)
  }

  async fn send_transfer(
    &self,
    account_id: &AccountIdentifier,
    transfer: &Transfer,
  ) -> Result<ResponseType> {
    let mut account = self.account_manager.get_account(*account_id)?;
    let synced = account.sync().execute().await?;
    let message = synced.transfer(transfer.clone()).await?;
    Ok(ResponseType::SentTransfer(message))
  }

  async fn internal_transfer(
    &self,
    from_account_id: &AccountIdentifier,
    to_account_id: &AccountIdentifier,
    amount: u64,
  ) -> Result<ResponseType> {
    let message = self
      .account_manager
      .internal_transfer(*from_account_id, *to_account_id, amount)
      .await?;
    Ok(ResponseType::SentTransfer(message))
  }
}

/// The Account actor.
pub struct Wallet {
  rx: UnboundedReceiver<Message>,
  message_handler: WalletMessageHandler,
}

impl Wallet {
  /// Runs the actor.
  pub async fn run(mut self) {
    println!("running wallet actor");

    while let Some(message) = self.rx.recv().await {
      self.message_handler.handle(message).await;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{
    message::{AccountToCreate, Message, MessageType, Response, ResponseType},
    WalletBuilder,
  };
  use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

  fn spawn_actor() -> UnboundedSender<Message> {
    let (tx, rx) = unbounded_channel();
    let actor = WalletBuilder::new().rx(rx).build();
    tokio::spawn(actor.run());
    tx
  }

  async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
    let (message_tx, mut message_rx) = unbounded_channel();
    let message = Message::new(0, message_type, message_tx);
    tx.send(message).unwrap();
    message_rx.recv().await.unwrap()
  }

  #[tokio::test]
  async fn create_and_remove_account() {
    let tx = spawn_actor();

    // create an account
    let account = AccountToCreate::default();
    send_message(
      &tx,
      MessageType::SetStrongholdPassword("password".to_string()),
    )
    .await;
    let response = send_message(&tx, MessageType::CreateAccount(account)).await;
    match response.response() {
      ResponseType::CreatedAccount(created_account) => {
        // remove the created account
        let response =
          send_message(&tx, MessageType::RemoveAccount(created_account.id().into())).await;
        assert!(matches!(
          response.response(),
          ResponseType::RemovedAccount(_)
        ));
      }
      _ => panic!("unexpected response"),
    }
  }
}

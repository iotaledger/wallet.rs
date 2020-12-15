// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountIdentifier,
    account_manager::AccountManager,
    message::{Message as WalletMessage, Transfer},
    DateTime, Result, Utc,
};
use futures::{Future, FutureExt};
use iota::message::prelude::MessageId;
use std::{
    any::Any,
    convert::TryInto,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    time::Duration,
};

mod message;
pub use message::*;

/// The Wallet message handler.
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
    Ok(ResponseType::Panic(format!("{}\n\n{:?}", msg, current_backtrace)))
}

fn convert_panics<F: FnOnce() -> Result<ResponseType>>(f: F) -> Result<ResponseType> {
    match catch_unwind(AssertUnwindSafe(|| f())) {
        Ok(result) => result,
        Err(panic) => panic_to_response_message(panic),
    }
}

async fn convert_async_panics<F>(f: impl FnOnce() -> F) -> Result<ResponseType>
where
    F: Future<Output = Result<ResponseType>>,
{
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => panic_to_response_message(panic),
    }
}

impl WalletMessageHandler {
    /// Creates a new instance of the message handler with the default account manager.
    pub fn new() -> Result<Self> {
        let instance = Self {
            account_manager: AccountManager::new()?,
        };
        Ok(instance)
    }

    /// Creates a new instance of the message handler with the account manager using the given storage path.
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

    /// Handles a message.
    pub async fn handle(&mut self, message: Message) {
        let response: Result<ResponseType> = match message.message_type() {
            MessageType::RemoveAccount(account_id) => convert_panics(|| self.remove_account(account_id)),
            MessageType::CreateAccount(account) => convert_panics(|| self.create_account(account)),
            MessageType::GetAccount(account_id) => convert_panics(|| self.get_account(account_id)),
            MessageType::GetAccounts => convert_panics(|| self.get_accounts()),
            MessageType::CallAccountMethod { account_id, method } => {
                convert_async_panics(|| async { self.call_account_method(account_id, method).await }).await
            }
            MessageType::SyncAccounts => convert_async_panics(|| async { self.sync_accounts().await }).await,
            MessageType::Reattach { account_id, message_id } => {
                convert_async_panics(|| async { self.reattach(account_id, message_id).await }).await
            }
            MessageType::Backup(destination_path) => convert_panics(|| self.backup(destination_path)),
            MessageType::RestoreBackup(backup_path) => convert_panics(|| self.restore_backup(backup_path)),
            MessageType::SetStrongholdPassword(password) => convert_panics(|| self.set_stronghold_password(password)),
            MessageType::SendTransfer { account_id, transfer } => {
                convert_async_panics(|| async { self.send_transfer(account_id, transfer).await }).await
            }
            MessageType::InternalTransfer {
                from_account_id,
                to_account_id,
                amount,
            } => {
                convert_async_panics(|| async { self.internal_transfer(from_account_id, to_account_id, *amount).await })
                    .await
            }
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => ResponseType::Error(e),
        };
        let _ = message
            .response_tx
            .send(Response::new(message.id().to_string(), message.message_type, response));
    }

    fn backup(&self, destination_path: &str) -> Result<ResponseType> {
        self.account_manager.backup(destination_path)?;
        Ok(ResponseType::BackupSuccessful)
    }

    fn restore_backup(&self, backup_path: &str) -> Result<ResponseType> {
        self.account_manager.import_accounts(backup_path)?;
        Ok(ResponseType::BackupRestored)
    }

    async fn reattach(&self, account_id: &AccountIdentifier, message_id: &str) -> Result<ResponseType> {
        let parsed_message_id = MessageId::new(
            message_id.as_bytes()[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid message id length"))?,
        );
        self.account_manager.reattach(account_id, &parsed_message_id).await?;
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
        let account = self.account_manager.get_account(account_id)?;

        match method {
            AccountMethod::GenerateAddress => {
                let mut account_ = account.write().unwrap();
                let address = account_.generate_address()?;
                Ok(ResponseType::GeneratedAddress(address))
            }
            AccountMethod::ListMessages {
                count,
                from,
                message_type,
            } => {
                let account_ = account.read().unwrap();
                let messages: Vec<WalletMessage> = account_
                    .list_messages(*count, *from, message_type.clone())
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(ResponseType::Messages(messages))
            }
            AccountMethod::ListAddresses { unspent } => {
                let account_ = account.read().unwrap();
                let addresses = account_.list_addresses(*unspent).into_iter().cloned().collect();
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::GetAvailableBalance => {
                let account_ = account.read().unwrap();
                Ok(ResponseType::AvailableBalance(account_.available_balance()))
            }
            AccountMethod::GetTotalBalance => {
                let account_ = account.read().unwrap();
                Ok(ResponseType::TotalBalance(account_.total_balance()))
            }
            AccountMethod::GetLatestAddress => {
                let account_ = account.read().unwrap();
                Ok(ResponseType::LatestAddress(account_.latest_address().cloned()))
            }
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
        self.account_manager
            .remove_account(&account_id)
            .map(|_| ResponseType::RemovedAccount(account_id.clone()))
    }

    /// The create account message handler.
    fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
        let mut builder = self.account_manager.create_account(account.client_options.clone());

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

        builder.initialise().map(|account| {
            let account = account.read().unwrap();
            ResponseType::CreatedAccount(account.clone())
        })
    }

    fn get_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
        let account = self.account_manager.get_account(&account_id)?;
        let account = account.read().unwrap();
        Ok(ResponseType::ReadAccount(account.clone()))
    }

    fn get_accounts(&self) -> Result<ResponseType> {
        let accounts = self.account_manager.get_accounts();
        let mut accounts_ = Vec::new();
        for account in accounts {
            let account_ = account.read().unwrap();
            accounts_.push(account_.clone());
        }
        Ok(ResponseType::ReadAccounts(accounts_))
    }

    fn set_stronghold_password(&mut self, password: &str) -> Result<ResponseType> {
        self.account_manager.set_stronghold_password(password)?;
        Ok(ResponseType::StrongholdPasswordSet)
    }

    async fn send_transfer(&self, account_id: &AccountIdentifier, transfer: &Transfer) -> Result<ResponseType> {
        let account = self.account_manager.get_account(account_id)?;
        let synced = account.sync().execute().await?;
        let message = synced.transfer(transfer.clone()).await?.message;
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
            .internal_transfer(from_account_id, to_account_id, amount)
            .await?
            .message;
        Ok(ResponseType::SentTransfer(message))
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler};
    use crate::client::ClientOptionsBuilder;
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

    /// The wallet actor builder.
    #[derive(Default)]
    pub struct WalletBuilder {
        rx: Option<UnboundedReceiver<Message>>,
        message_handler: Option<WalletMessageHandler>,
    }

    impl WalletBuilder {
        /// Creates a new wallet actor builder.
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets the receiver for messages.
        pub fn rx(mut self, rx: UnboundedReceiver<Message>) -> Self {
            self.rx.replace(rx);
            self
        }

        /// Sets the wallet message handler
        pub fn message_handler(mut self, message_handler: WalletMessageHandler) -> Self {
            self.message_handler.replace(message_handler);
            self
        }

        /// Builds the Wallet actor.
        pub fn build(self) -> Wallet {
            Wallet {
                rx: self.rx.expect("rx is required"),
                message_handler: self
                    .message_handler
                    .unwrap_or_else(|| WalletMessageHandler::new().expect("failed to initialise account manager")),
            }
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

    fn spawn_actor() -> UnboundedSender<Message> {
        let (tx, rx) = unbounded_channel();
        let actor = WalletBuilder::new().rx(rx).build();
        std::thread::spawn(|| {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(actor.run());
        });
        tx
    }

    async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
        let (message_tx, mut message_rx) = unbounded_channel();
        let message = Message::new("".to_string(), message_type, message_tx);
        tx.send(message).unwrap();
        message_rx.recv().await.unwrap()
    }

    #[tokio::test]
    async fn create_and_remove_account() {
        let tx = spawn_actor();

        // create an account
        let account = AccountToCreate {
            client_options: ClientOptionsBuilder::node("http://node.iota").unwrap().build(),
            mnemonic: None,
            alias: None,
            created_at: None,
        };
        send_message(&tx, MessageType::SetStrongholdPassword("password".to_string())).await;
        let response = send_message(&tx, MessageType::CreateAccount(account)).await;
        match response.response() {
            ResponseType::CreatedAccount(created_account) => {
                let id = created_account.id().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(6));
                    // remove the created account
                    let response =
                        crate::block_on(async move { send_message(&tx, MessageType::RemoveAccount(id)).await });
                    assert!(matches!(response.response(), ResponseType::RemovedAccount(_)));
                });
            }
            _ => panic!("unexpected response"),
        }
    }
}

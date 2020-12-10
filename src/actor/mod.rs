// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountIdentifier},
    account_manager::AccountManager,
    message::Transfer,
    DateTime, Result, Utc,
};
use futures::{Future, FutureExt};
use iota::message::prelude::MessageId;
use std::{
    any::Any,
    collections::HashMap,
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
    // TODO use this as a proper cache mechanism
    accounts: HashMap<AccountIdentifier, Account>,
}

impl Default for WalletMessageHandler {
    fn default() -> Self {
        Self {
            account_manager: AccountManager::new().unwrap(),
            accounts: Default::default(),
        }
    }
}

fn panic_to_response_message<'a>(panic: Box<dyn Any>) -> Result<ResponseType<'a>> {
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

fn convert_panics<'a, F: FnOnce() -> Result<ResponseType<'a>>>(f: F) -> Result<ResponseType<'a>> {
    match catch_unwind(AssertUnwindSafe(|| f())) {
        Ok(result) => result,
        Err(panic) => panic_to_response_message(panic),
    }
}

async fn convert_async_panics<'a, F>(f: impl FnOnce() -> F) -> Result<ResponseType<'a>>
where
    F: Future<Output = Result<ResponseType<'a>>>,
{
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => panic_to_response_message(panic),
    }
}

impl<'a> WalletMessageHandler {
    /// Creates a new instance of the message handler with the default account manager.
    pub fn new() -> Result<Self> {
        let instance = Self {
            account_manager: AccountManager::new()?,
            accounts: Default::default(),
        };
        Ok(instance)
    }

    /// Creates a new instance of the message handler with the account manager using the given storage path.
    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        let instance = Self {
            account_manager: AccountManager::with_storage_path(storage_path)?,
            accounts: Default::default(),
        };
        Ok(instance)
    }

    /// Gets the account manager instance.
    pub fn set_polling_interval(&mut self, interval: Duration) {
        self.account_manager.set_polling_interval(interval);
    }

    /// Handles a message.
    pub async fn handle(&'a mut self, message: Message<'a>) {
        let response: Result<ResponseType<'_>> = match message.message_type() {
            MessageType::RemoveAccount(account_id) => convert_panics(move || self.remove_account(account_id)),
            MessageType::CreateAccount(account) => convert_panics(move || self.create_account(account)),
            MessageType::GetAccount(account_id) => convert_panics(move || self.get_account(account_id)),
            MessageType::GetAccounts => convert_panics(move || self.get_accounts()),
            MessageType::CallAccountMethod { account_id, method } => {
                convert_async_panics(move || async move { self.call_account_method(account_id, method).await }).await
            }
            MessageType::SyncAccounts => convert_async_panics(move || async move { self.sync_accounts().await }).await,
            MessageType::Reattach { account_id, message_id } => {
                convert_async_panics(move || async move { self.reattach(account_id, message_id).await }).await
            }
            MessageType::Backup(destination_path) => convert_panics(move || self.backup(destination_path)),
            MessageType::RestoreBackup(backup_path) => convert_panics(move || self.restore_backup(backup_path)),
            MessageType::SetStrongholdPassword(password) => {
                convert_panics(move || self.set_stronghold_password(password))
            }
            MessageType::SendTransfer { account_id, transfer } => {
                convert_async_panics(move || async move { self.send_transfer(account_id, transfer).await }).await
            }
            MessageType::InternalTransfer {
                from_account_id,
                to_account_id,
                amount,
            } => {
                let from_account_id = from_account_id.clone();
                let to_account_id = to_account_id.clone();
                let amount = *amount;
                convert_async_panics(move || async move {
                    self.internal_transfer(&from_account_id, &to_account_id, amount).await
                })
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

    fn backup(&self, destination_path: &str) -> Result<ResponseType<'_>> {
        self.account_manager.backup(destination_path)?;
        Ok(ResponseType::BackupSuccessful)
    }

    fn restore_backup(&self, backup_path: &str) -> Result<ResponseType<'_>> {
        self.account_manager.import_accounts(backup_path)?;
        Ok(ResponseType::BackupRestored)
    }

    async fn reattach(&self, account_id: &AccountIdentifier, message_id: &str) -> Result<ResponseType<'_>> {
        let parsed_message_id = MessageId::new(
            message_id.as_bytes()[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid message id length"))?,
        );
        self.account_manager.reattach(account_id, &parsed_message_id).await?;
        Ok(ResponseType::Reattached(message_id.to_string()))
    }

    async fn sync_accounts(&self) -> Result<ResponseType<'_>> {
        let synced = self.account_manager.sync_accounts().await?;
        Ok(ResponseType::SyncedAccounts(synced))
    }

    async fn call_account_method(
        &mut self,
        account_id: &AccountIdentifier,
        method: &AccountMethod,
    ) -> Result<ResponseType<'_>> {
        let account = self.account_manager.get_account(account_id)?;
        self.accounts.insert(account_id.clone(), account);
        let account = self.accounts.get_mut(account_id).unwrap();

        match method {
            AccountMethod::GenerateAddress => {
                let address = account.generate_address()?;
                Ok(ResponseType::GeneratedAddress(address))
            }
            AccountMethod::ListMessages {
                count,
                from,
                message_type,
            } => {
                let messages = account.list_messages(*count, *from, message_type.clone());
                Ok(ResponseType::Messages(messages))
            }
            AccountMethod::ListAddresses { unspent } => {
                let addresses = account.list_addresses(*unspent);
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::GetAvailableBalance => Ok(ResponseType::AvailableBalance(account.available_balance())),
            AccountMethod::GetTotalBalance => Ok(ResponseType::TotalBalance(account.total_balance())),
            AccountMethod::GetLatestAddress => Ok(ResponseType::LatestAddress(account.latest_address())),
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
    fn remove_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType<'_>> {
        self.account_manager
            .remove_account(&account_id)
            .map(|_| ResponseType::RemovedAccount(account_id.clone()))
    }

    /// The create account message handler.
    fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType<'_>> {
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

        builder.initialise().map(ResponseType::CreatedAccount)
    }

    fn get_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType<'_>> {
        let account = self.account_manager.get_account(&account_id)?;
        Ok(ResponseType::ReadAccount(account))
    }

    fn get_accounts(&mut self) -> Result<ResponseType<'_>> {
        let accounts = self.account_manager.get_accounts()?;
        for account in accounts {
            self.accounts.insert(account.id().clone(), account);
        }
        Ok(ResponseType::ReadAccounts(self.accounts.values().collect()))
    }

    fn set_stronghold_password(&mut self, password: &str) -> Result<ResponseType<'_>> {
        self.account_manager.set_stronghold_password(password)?;
        Ok(ResponseType::StrongholdPasswordSet)
    }

    async fn send_transfer(&self, account_id: &AccountIdentifier, transfer: &Transfer) -> Result<ResponseType<'_>> {
        let mut account = self.account_manager.get_account(account_id)?;
        let synced = account.sync().execute().await?;
        let message = synced.transfer(transfer.clone()).await?.message;
        Ok(ResponseType::SentTransfer(message))
    }

    async fn internal_transfer(
        &self,
        from_account_id: &AccountIdentifier,
        to_account_id: &AccountIdentifier,
        amount: u64,
    ) -> Result<ResponseType<'_>> {
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
                message_handler: WalletMessageHandler::new().expect("failed to initialise account manager"),
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
        let account = AccountToCreate::default();
        send_message(&tx, MessageType::SetStrongholdPassword("password".to_string())).await;
        let response = send_message(&tx, MessageType::CreateAccount(account)).await;
        match response.response() {
            ResponseType::CreatedAccount(created_account) => {
                // remove the created account
                let response = send_message(&tx, MessageType::RemoveAccount(created_account.id().clone())).await;
                assert!(matches!(response.response(), ResponseType::RemovedAccount(_)));
            }
            _ => panic!("unexpected response"),
        }
    }
}

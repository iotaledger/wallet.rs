// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::{
    account::{
        operations::transfer::{TransferOptions, TransferOutput, TransferResult},
        types::AccountIdentifier,
    },
    account_manager::AccountManager,
    Result,
};

use backtrace::Backtrace;
use futures::{Future, FutureExt};
use zeroize::Zeroize;

use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
};

mod message;
pub use message::*;

/// The Wallet message handler.
pub struct WalletMessageHandler {
    account_manager: AccountManager,
}

fn panic_to_response_message(panic: Box<dyn Any>) -> ResponseType {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = Backtrace::new();
    ResponseType::Panic(format!("{}\n\n{:?}", msg, current_backtrace))
}

fn convert_panics<F: FnOnce() -> Result<ResponseType>>(f: F) -> Result<ResponseType> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic) => Ok(panic_to_response_message(panic)),
    }
}

async fn convert_async_panics<F>(f: impl FnOnce() -> F) -> Result<ResponseType>
where
    F: Future<Output = Result<ResponseType>>,
{
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Ok(panic_to_response_message(panic)),
    }
}

impl WalletMessageHandler {
    /// Creates a new instance of the message handler with the default account manager.
    pub async fn new() -> Result<Self> {
        let instance = Self {
            account_manager: AccountManager::builder().finish().await?,
        };
        Ok(instance)
    }

    /// Creates a new instance of the message handler with the specified account manager.
    pub fn with_manager(account_manager: AccountManager) -> Self {
        Self { account_manager }
    }

    /// Handles a message.
    pub async fn handle(&self, mut message: Message) {
        let response: Result<ResponseType> = match message.message_type_mut() {
            MessageType::CreateAccount(account) => {
                convert_async_panics(|| async { self.create_account(account).await }).await
            }
            MessageType::GetAccount(account_id) => {
                convert_async_panics(|| async { self.get_account(account_id).await }).await
            }
            MessageType::GetAccounts => convert_async_panics(|| async { self.get_accounts().await }).await,
            MessageType::CallAccountMethod { account_id, method } => {
                convert_async_panics(|| async { self.call_account_method(account_id, method).await }).await
            }
            #[cfg(feature = "storage")]
            MessageType::Backup { destination, password } => {
                convert_async_panics(|| async {
                    let res = self.backup(destination, password.to_string()).await;
                    password.zeroize();
                    res
                })
                .await
            }
            #[cfg(feature = "storage")]
            MessageType::RestoreBackup { source, password } => {
                let res =
                    convert_async_panics(|| async { self.restore_backup(source, password.to_string()).await }).await;
                password.zeroize();
                res
            }
            #[cfg(feature = "storage")]
            MessageType::DeleteStorage => {
                convert_async_panics(|| async {
                    self.account_manager.delete_storage().await?;
                    Ok(ResponseType::DeletedStorage)
                })
                .await
            }
            MessageType::GenerateMnemonic => convert_panics(|| {
                self.account_manager
                    .generate_mnemonic()
                    .map(ResponseType::GeneratedMnemonic)
            }),
            MessageType::VerifyMnemonic(mnemonic) => convert_panics(|| {
                self.account_manager
                    .verify_mnemonic(mnemonic)
                    .map(|_| ResponseType::VerifiedMnemonic)
            }),
            MessageType::SendTransfer {
                account_id,
                outputs,
                options,
            } => {
                convert_async_panics(|| async {
                    self.send_transfer(account_id, outputs.clone(), options.clone()).await
                })
                .await
            }
            MessageType::SetClientOptions(options) => {
                convert_async_panics(|| async {
                    self.account_manager.set_client_options(*options.clone()).await?;
                    Ok(ResponseType::UpdatedAllClientOptions)
                })
                .await
            }
            MessageType::StartBackgroundSync { options, interval } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .start_background_syncing(options.clone(), *interval)
                        .await?;
                    Ok(ResponseType::Ok(()))
                })
                .await
            }
            MessageType::StopBackgroundSync => {
                convert_async_panics(|| async {
                    self.account_manager.stop_background_syncing()?;
                    Ok(ResponseType::Ok(()))
                })
                .await
            }
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => ResponseType::Error(e),
        };
        let _ = message.response_tx.send(Response::new(message.message_type, response));
    }

    #[cfg(feature = "storage")]
    async fn backup(&self, destination_path: &Path, password: String) -> Result<ResponseType> {
        self.account_manager.backup(destination_path, password).await?;
        Ok(ResponseType::BackupSuccessful)
    }

    #[cfg(feature = "storage")]
    async fn restore_backup(&self, source: &str, password: String) -> Result<ResponseType> {
        self.account_manager.restore_backup(source, password).await?;
        Ok(ResponseType::BackupRestored)
    }

    async fn call_account_method(
        &self,
        account_id: &AccountIdentifier,
        method: &AccountMethod,
    ) -> Result<ResponseType> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;

        match method {
            AccountMethod::GenerateAddresses { amount, options } => {
                let address = account_handle.generate_addresses(*amount, options.clone()).await?;
                Ok(ResponseType::GeneratedAddress(address))
            }
            AccountMethod::ListAddresses => {
                let addresses = account_handle.list_addresses().await?;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListAddressesWithBalance => {
                let addresses = account_handle.list_addresses_with_balance().await?;
                Ok(ResponseType::AddressesWithBalance(addresses))
            }
            AccountMethod::ListOutputs => {
                let outputs = account_handle.list_outputs().await?;
                Ok(ResponseType::Outputs(outputs))
            }
            AccountMethod::ListUnspentOutputs => {
                let outputs = account_handle.list_unspent_outputs().await?;
                Ok(ResponseType::Outputs(outputs))
            }
            AccountMethod::ListTransactions => {
                let transactions = account_handle.list_transactions().await?;
                Ok(ResponseType::Transactions(transactions))
            }
            AccountMethod::ListPendingTransactions => {
                let transactions = account_handle.list_pending_transactions().await?;
                Ok(ResponseType::Transactions(transactions))
            }
            AccountMethod::GetBalance => Ok(ResponseType::Balance(account_handle.balance().await?)),
            AccountMethod::SyncAccount { options } => {
                Ok(ResponseType::Balance(account_handle.sync(options.clone()).await?))
            }
        }
    }

    /// The create account message handler.
    async fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
        let mut builder = self.account_manager.create_account();

        if let Some(alias) = &account.alias {
            builder = builder.with_alias(alias.clone());
        }

        match builder.finish().await {
            Ok(account_handle) => {
                let account = account_handle.read().await;
                Ok(ResponseType::CreatedAccount(account.clone()))
            }
            Err(e) => Err(e),
        }
    }

    async fn get_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;
        let account = account_handle.read().await;
        Ok(ResponseType::ReadAccount(account.clone()))
    }

    async fn get_accounts(&self) -> Result<ResponseType> {
        let account_handles = self.account_manager.get_accounts().await?;
        let mut accounts = Vec::new();
        for account_handle in account_handles {
            let account = account_handle.read().await;
            accounts.push(account.clone());
        }
        Ok(ResponseType::ReadAccounts(accounts))
    }

    async fn send_transfer(
        &self,
        account_id: &AccountIdentifier,
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> Result<ResponseType> {
        let account = self.account_manager.get_account(account_id.clone()).await?;
        let message = account.send(outputs, options).await?;
        Ok(ResponseType::SentTransfer(message))
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler};
    use crate::account_manager::AccountManager;
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
        pub async fn build(self) -> Wallet {
            Wallet {
                rx: self.rx.expect("rx is required"),
                message_handler: self.message_handler.expect("message handler is required"),
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

    fn spawn_actor(manager: AccountManager) -> UnboundedSender<Message> {
        let (tx, rx) = unbounded_channel();
        std::thread::spawn(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                let actor = WalletBuilder::new()
                    .rx(rx)
                    .message_handler(WalletMessageHandler::with_manager(manager))
                    .build()
                    .await;
                actor.run().await
            });
        });
        tx
    }

    async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
        let (message_tx, mut message_rx) = unbounded_channel();
        let message = Message::new(message_type, message_tx);
        tx.send(message).unwrap();
        message_rx.recv().await.unwrap()
    }

    #[tokio::test]
    async fn create_account() {
        let manager = AccountManager::builder().finish().await.unwrap();
        let tx = spawn_actor(manager);

        // create an account
        let account = AccountToCreate { alias: None };
        let response = send_message(&tx, MessageType::CreateAccount(Box::new(account))).await;
        match response.response() {
            ResponseType::CreatedAccount(account) => {
                let id = account.id().clone();
                println!("Created account id: {id}")
            }
            _ => panic!("unexpected response {:?}", response),
        }
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountIdentifier,
    account_manager::AccountManager,
    message::{Message as WalletMessage, Transfer},
    Result,
};
use futures::{Future, FutureExt};
use iota::message::prelude::MessageId;
use zeroize::Zeroize;

use std::{
    any::Any,
    convert::TryInto,
    num::NonZeroU64,
    panic::{catch_unwind, AssertUnwindSafe},
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
    let current_backtrace = backtrace::Backtrace::new();
    ResponseType::Panic(format!("{}\n\n{:?}", msg, current_backtrace))
}

fn convert_panics<F: FnOnce() -> Result<ResponseType>>(f: F) -> Result<ResponseType> {
    match catch_unwind(AssertUnwindSafe(|| f())) {
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
    pub async fn handle(&mut self, mut message: Message) {
        let response: Result<ResponseType> = match message.message_type_mut() {
            MessageType::RemoveAccount(account_id) => {
                convert_async_panics(|| async { self.remove_account(account_id).await }).await
            }
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
            MessageType::SyncAccounts {
                address_index,
                gap_limit,
            } => convert_async_panics(|| async { self.sync_accounts(address_index, gap_limit).await }).await,
            MessageType::Reattach { account_id, message_id } => {
                convert_async_panics(|| async { self.reattach(account_id, message_id).await }).await
            }
            #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
            MessageType::Backup(destination_path) => {
                convert_async_panics(|| async { self.backup(destination_path).await }).await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::RestoreBackup { backup_path, password } => {
                let res =
                    convert_async_panics(|| async { self.restore_backup(backup_path, password.to_string()).await })
                        .await;
                password.zeroize();
                res
            }
            #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
            MessageType::RestoreBackup { backup_path } => {
                convert_async_panics(|| async { self.restore_backup(backup_path).await }).await
            }
            MessageType::SetStoragePassword(password) => {
                convert_async_panics(|| async { self.set_storage_password(password).await }).await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::SetStrongholdPassword(password) => {
                convert_async_panics(|| async { self.set_stronghold_password(password).await }).await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::SetStrongholdPasswordClearInterval(interval) => {
                convert_async_panics(|| async {
                    crate::set_stronghold_password_clear_interval(*interval).await;
                    Ok(ResponseType::StrongholdPasswordClearIntervalSet)
                })
                .await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::GetStrongholdStatus => {
                convert_async_panics(|| async {
                    let status =
                        crate::get_stronghold_status(&self.account_manager.stronghold_snapshot_path().await?).await;
                    Ok(ResponseType::StrongholdStatus(status))
                })
                .await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::LockStronghold => {
                convert_async_panics(|| async {
                    crate::lock_stronghold(&self.account_manager.stronghold_snapshot_path().await?, true).await?;
                    Ok(ResponseType::LockedStronghold)
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
            MessageType::StoreMnemonic { signer_type, mnemonic } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .store_mnemonic(signer_type.clone(), mnemonic.clone())
                        .await
                        .map(|_| ResponseType::StoredMnemonic)
                })
                .await
            }
            MessageType::IsLatestAddressUnused => {
                convert_async_panics(|| async {
                    self.account_manager
                        .is_latest_address_unused()
                        .await
                        .map(ResponseType::AreAllLatestAddressesUnused)
                })
                .await
            }
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            MessageType::GetLedgerStatus(is_simulator) => {
                convert_panics(|| Ok(ResponseType::LedgerStatus(crate::get_ledger_status(*is_simulator))))
            }
            MessageType::DeleteStorage => {
                convert_async_panics(|| async move {
                    self.account_manager.delete_internal().await?;
                    Ok(ResponseType::DeletedStorage)
                })
                .await
            }
            MessageType::SendTransfer { account_id, transfer } => {
                convert_async_panics(|| async { self.send_transfer(account_id, transfer.clone().finish()).await }).await
            }
            MessageType::InternalTransfer {
                from_account_id,
                to_account_id,
                amount,
            } => {
                convert_async_panics(|| async { self.internal_transfer(from_account_id, to_account_id, *amount).await })
                    .await
            }
            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            MessageType::ChangeStrongholdPassword {
                current_password,
                new_password,
            } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .change_stronghold_password(current_password.to_string(), new_password.to_string())
                        .await?;
                    current_password.zeroize();
                    new_password.zeroize();
                    Ok(ResponseType::StrongholdPasswordChanged)
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
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => ResponseType::Error(e),
        };
        let _ = message
            .response_tx
            .send(Response::new(message.id().to_string(), message.message_type, response));
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    async fn backup(&self, destination_path: &str) -> Result<ResponseType> {
        self.account_manager.backup(destination_path).await?;
        Ok(ResponseType::BackupSuccessful)
    }

    #[cfg(any(feature = "stronghold-storage", feature = "sqlite-storage"))]
    async fn restore_backup(
        &mut self,
        backup_path: &str,
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))] password: String,
    ) -> Result<ResponseType> {
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        self.account_manager.import_accounts(backup_path, password).await?;
        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        self.account_manager.import_accounts(backup_path).await?;
        Ok(ResponseType::BackupRestored)
    }

    async fn reattach(&self, account_id: &AccountIdentifier, message_id: &str) -> Result<ResponseType> {
        let parsed_message_id = MessageId::new(
            message_id.as_bytes()[..]
                .try_into()
                .map_err(|_| crate::Error::InvalidMessageId)?,
        );
        self.account_manager
            .reattach(account_id.clone(), &parsed_message_id)
            .await?;
        Ok(ResponseType::Reattached(message_id.to_string()))
    }

    async fn sync_accounts(&self, address_index: &Option<usize>, gap_limit: &Option<usize>) -> Result<ResponseType> {
        let mut synchronizer = self.account_manager.sync_accounts()?;
        if let Some(address_index) = address_index {
            synchronizer = synchronizer.address_index(*address_index);
        }
        if let Some(gap_limit) = gap_limit {
            synchronizer = synchronizer.gap_limit(*gap_limit);
        }
        let synced = synchronizer.execute().await?;
        Ok(ResponseType::SyncedAccounts(synced))
    }

    async fn call_account_method(
        &self,
        account_id: &AccountIdentifier,
        method: &AccountMethod,
    ) -> Result<ResponseType> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;

        match method {
            AccountMethod::GenerateAddress => {
                let address = account_handle.generate_address().await?;
                Ok(ResponseType::GeneratedAddress(address))
            }
            AccountMethod::GetUnusedAddress => {
                let address = account_handle.get_unused_address().await?;
                Ok(ResponseType::UnusedAddress(address))
            }
            AccountMethod::ListMessages {
                count,
                from,
                message_type,
            } => {
                let messages: Vec<WalletMessage> = account_handle
                    .read()
                    .await
                    .list_messages(*count, *from, message_type.clone())
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(ResponseType::Messages(messages))
            }
            AccountMethod::ListAddresses => {
                let addresses = account_handle.addresses().await;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListSpentAddresses => {
                let addresses = account_handle.list_spent_addresses().await;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListUnspentAddresses => {
                let addresses = account_handle.list_unspent_addresses().await;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::GetBalance => Ok(ResponseType::Balance(account_handle.read().await.balance())),
            AccountMethod::GetLatestAddress => Ok(ResponseType::LatestAddress(
                account_handle.read().await.latest_address().clone(),
            )),
            AccountMethod::SyncAccount {
                address_index,
                gap_limit,
                skip_persistance,
            } => {
                let mut synchronizer = account_handle.sync().await;
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
            AccountMethod::IsLatestAddressUnused => Ok(ResponseType::IsLatestAddressUnused(
                account_handle.is_latest_address_unused().await?,
            )),
            AccountMethod::SetAlias(alias) => {
                account_handle.set_alias(alias).await?;
                Ok(ResponseType::UpdatedAlias)
            }
            AccountMethod::SetClientOptions(options) => {
                account_handle.set_client_options(*options.clone()).await?;
                Ok(ResponseType::UpdatedClientOptions)
            }
        }
    }

    /// The remove account message handler.
    async fn remove_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
        self.account_manager
            .remove_account(account_id.clone())
            .await
            .map(|_| ResponseType::RemovedAccount(account_id.clone()))
    }

    /// The create account message handler.
    async fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
        let mut builder = self.account_manager.create_account(account.client_options.clone())?;

        if let Some(alias) = &account.alias {
            builder = builder.alias(alias);
        }
        if let Some(created_at) = &account.created_at {
            builder = builder.created_at(*created_at);
        }
        if account.skip_persistance {
            builder = builder.skip_persistance();
        }
        if let Some(signer_type) = &account.signer_type {
            builder = builder.signer_type(signer_type.clone());
        }

        match builder.initialise().await {
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
        let accounts = self.account_manager.get_accounts().await?;
        let mut accounts_ = Vec::new();
        for account_handle in accounts {
            accounts_.push(account_handle.read().await.clone());
        }
        Ok(ResponseType::ReadAccounts(accounts_))
    }

    async fn set_storage_password(&mut self, password: &str) -> Result<ResponseType> {
        self.account_manager.set_storage_password(password).await?;
        Ok(ResponseType::StoragePasswordSet)
    }

    #[cfg(feature = "stronghold")]
    async fn set_stronghold_password(&mut self, password: &str) -> Result<ResponseType> {
        self.account_manager.set_stronghold_password(password).await?;
        Ok(ResponseType::StrongholdPasswordSet)
    }

    async fn send_transfer(&self, account_id: &AccountIdentifier, transfer: Transfer) -> Result<ResponseType> {
        let account = self.account_manager.get_account(account_id.clone()).await?;
        let message = account.transfer(transfer).await?;
        Ok(ResponseType::SentTransfer(message))
    }

    async fn internal_transfer(
        &self,
        from_account_id: &AccountIdentifier,
        to_account_id: &AccountIdentifier,
        amount: NonZeroU64,
    ) -> Result<ResponseType> {
        let message = self
            .account_manager
            .internal_transfer(from_account_id.clone(), to_account_id.clone(), amount)
            .await?;
        Ok(ResponseType::SentTransfer(message))
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler};
    use crate::{account_manager::AccountManager, client::ClientOptionsBuilder};
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
        let message = Message::new("".to_string(), message_type, message_tx);
        tx.send(message).unwrap();
        message_rx.recv().await.unwrap()
    }

    #[tokio::test]
    async fn create_and_remove_account() {
        crate::test_utils::with_account_manager(
            crate::test_utils::TestType::SigningAndStorage,
            |manager, signer_type| async move {
                let tx = spawn_actor(manager);

                // create an account
                let account = AccountToCreate {
                    client_options: ClientOptionsBuilder::new()
                        .with_node("http://api.hornet-1.testnet.chrysalis2.com/")
                        .unwrap()
                        .build()
                        .unwrap(),
                    alias: None,
                    created_at: None,
                    skip_persistance: false,
                    signer_type: Some(signer_type.clone()),
                };
                #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
                send_message(&tx, MessageType::SetStrongholdPassword("password".to_string())).await;
                send_message(
                    &tx,
                    MessageType::StoreMnemonic {
                        signer_type,
                        mnemonic: None,
                    },
                )
                .await;
                let response = send_message(&tx, MessageType::CreateAccount(Box::new(account))).await;
                match response.response() {
                    ResponseType::CreatedAccount(created_account) => {
                        let id = created_account.id().clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(6));
                            // remove the created account
                            let response = crate::block_on(async move {
                                send_message(&tx, MessageType::RemoveAccount(id.into())).await
                            });
                            assert!(matches!(response.response(), ResponseType::RemovedAccount(_)));
                        });
                    }
                    _ => panic!("unexpected response {:?}", response),
                }
            },
        )
        .await;
    }
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
};

use backtrace::Backtrace;
use futures::{Future, FutureExt};
use zeroize::Zeroize;

#[cfg(feature = "events")]
use crate::events::types::{Event, WalletEventType};
use crate::{
    account::types::AccountIdentifier,
    account_manager::AccountManager,
    message_interface::{
        account_method::AccountMethod,
        message::Message,
        message_type::{AccountToCreate, MessageType},
        response::Response,
        response_type::ResponseType,
    },
    Result,
};

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

/// The Wallet message handler.
pub struct WalletMessageHandler {
    account_manager: AccountManager,
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

    #[cfg(feature = "events")]
    /// Listen to wallet events, empty vec will listen to all events
    pub async fn listen<F>(&self, events: Vec<WalletEventType>, handler: F)
    where
        F: Fn(&Event) + 'static + Clone + Send + Sync,
    {
        self.account_manager.listen(events, handler).await;
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
            MessageType::SetClientOptions(options) => {
                convert_async_panics(|| async {
                    self.account_manager.set_client_options(*options.clone()).await?;
                    Ok(ResponseType::UpdatedAllClientOptions)
                })
                .await
            }
            MessageType::GetNodeInfo => {
                convert_async_panics(|| async {
                    self.account_manager.get_node_info().await.map(ResponseType::NodeInfo)
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
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            MessageType::EmitTestEvent(event) => {
                convert_async_panics(|| async {
                    self.account_manager.emit_test_event(event.clone()).await?;
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
            AccountMethod::GetOutputsWithAdditionalUnlockConditions { outputs_to_collect } => {
                let output_ids = account_handle
                    .get_unlockable_outputs_with_additional_unlock_conditions(*outputs_to_collect)
                    .await?;
                Ok(ResponseType::OutputIds(output_ids))
            }
            AccountMethod::ListAddresses => {
                let addresses = account_handle.list_addresses().await?;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListAddressesWithUnspentOutputs => {
                let addresses = account_handle.list_addresses_with_unspent_outputs().await?;
                Ok(ResponseType::AddressesWithUnspentOutputs(addresses))
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
            AccountMethod::MintNativeToken {
                native_token_options,
                options,
            } => {
                convert_async_panics(|| async {
                    let message = account_handle
                        .mint_native_token(native_token_options.clone(), options.clone())
                        .await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::MintNfts { nfts_options, options } => {
                convert_async_panics(|| async {
                    let message = account_handle.mint_nfts(nfts_options.clone(), options.clone()).await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::GetBalance => Ok(ResponseType::Balance(account_handle.balance().await?)),
            AccountMethod::SyncAccount { options } => {
                Ok(ResponseType::Balance(account_handle.sync(options.clone()).await?))
            }
            AccountMethod::SendAmount {
                addresses_with_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let message = account_handle
                        .send_amount(addresses_with_amount.clone(), options.clone())
                        .await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::SendMicroTransaction {
                addresses_with_micro_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let message = account_handle
                        .send_micro_transaction(addresses_with_micro_amount.clone(), options.clone())
                        .await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::SendNativeTokens {
                addresses_native_tokens,
                options,
            } => {
                convert_async_panics(|| async {
                    let message = account_handle
                        .send_native_tokens(addresses_native_tokens.clone(), options.clone())
                        .await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::SendNft {
                addresses_nft_ids,
                options,
            } => {
                convert_async_panics(|| async {
                    let message = account_handle
                        .send_nft(addresses_nft_ids.clone(), options.clone())
                        .await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::SendTransfer { outputs, options } => {
                convert_async_panics(|| async {
                    let message = account_handle.send(outputs.clone(), options.clone()).await?;
                    Ok(ResponseType::SentTransfer(message))
                })
                .await
            }
            AccountMethod::TryCollectOutputs { outputs_to_collect } => {
                convert_async_panics(|| async {
                    let transfer_results = account_handle.try_collect_outputs(*outputs_to_collect).await?;
                    Ok(ResponseType::SentTransfers(transfer_results))
                })
                .await
            }
            AccountMethod::CollectOutputs { output_ids_to_collect } => {
                convert_async_panics(|| async {
                    let transfer_results = account_handle.collect_outputs(output_ids_to_collect.to_vec()).await?;
                    Ok(ResponseType::SentTransfers(transfer_results))
                })
                .await
            }
        }
    }

    /// The create account message handler.
    async fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
        let mut builder = self.account_manager.create_account();

        if let Some(alias) = &account.alias {
            builder = builder.with_alias(alias.clone());
        }

        if let Some(coin_type) = &account.coin_type {
            builder = builder.with_coin_type((*coin_type).try_into()?);
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
}

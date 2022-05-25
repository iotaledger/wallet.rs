// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
};

use backtrace::Backtrace;
use futures::{Future, FutureExt};
use iota_client::{
    api::{PreparedTransactionData, PreparedTransactionDataDto, SignedTransactionData, SignedTransactionDataDto},
    bee_block::output::Output,
    Client, NodeInfoWrapper,
};
use tokio::sync::mpsc::UnboundedSender;
use zeroize::Zeroize;

#[cfg(feature = "events")]
use crate::events::types::{Event, WalletEventType};
use crate::{
    account::types::AccountIdentifier,
    account_manager::AccountManager,
    message_interface::{
        account_method::AccountMethod,
        dtos::{AccountBalanceDto, AccountDto, OutputDataDto, TransactionDto},
        message::{AccountToCreate, Message},
        response::Response,
        AddressWithUnspentOutputsDto,
    },
    AddressWithAmount, AddressWithMicroAmount, Result,
};

fn panic_to_response_message(panic: Box<dyn Any>) -> Response {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = Backtrace::new();
    Response::Panic(format!("{}\n\n{:?}", msg, current_backtrace))
}

fn convert_panics<F: FnOnce() -> Result<Response>>(f: F) -> Result<Response> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic) => Ok(panic_to_response_message(panic)),
    }
}

async fn convert_async_panics<F>(f: impl FnOnce() -> F) -> Result<Response>
where
    F: Future<Output = Result<Response>>,
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
    pub async fn handle(&self, message: Message, response_tx: UnboundedSender<Response>) {
        log::debug!("Message: {:?}", message);

        let response: Result<Response> = match message {
            Message::CreateAccount(account) => {
                convert_async_panics(|| async { self.create_account(&account).await }).await
            }
            Message::GetAccount(account_id) => {
                convert_async_panics(|| async { self.get_account(&account_id).await }).await
            }
            Message::GetAccounts => convert_async_panics(|| async { self.get_accounts().await }).await,
            Message::CallAccountMethod { account_id, method } => {
                convert_async_panics(|| async { self.call_account_method(&account_id, &method).await }).await
            }
            #[cfg(feature = "stronghold")]
            Message::Backup { destination, password } => {
                convert_async_panics(|| async { self.backup(destination.to_path_buf(), password).await }).await
            }
            #[cfg(feature = "stronghold")]
            Message::ClearStrongholdPassword => {
                convert_async_panics(|| async {
                    self.account_manager.clear_stronghold_password().await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            #[cfg(feature = "stronghold")]
            Message::IsStrongholdPasswordAvailable => {
                convert_async_panics(|| async {
                    let is_available = self.account_manager.is_stronghold_password_available().await?;
                    Ok(Response::StrongholdPasswordIsAvailable(is_available))
                })
                .await
            }
            Message::RecoverAccounts {
                account_gap_limit,
                address_gap_limit,
            } => {
                convert_async_panics(|| async {
                    let account_handles = self
                        .account_manager
                        .recover_accounts(account_gap_limit, address_gap_limit)
                        .await?;
                    let mut accounts = Vec::new();
                    for account_handle in account_handles {
                        let account = account_handle.read().await;
                        accounts.push(AccountDto::from(&*account));
                    }
                    Ok(Response::Accounts(accounts))
                })
                .await
            }
            #[cfg(feature = "stronghold")]
            Message::RestoreBackup { source, password } => {
                convert_async_panics(|| async { self.restore_backup(source.to_path_buf(), password).await }).await
            }
            #[cfg(feature = "storage")]
            Message::DeleteStorage => {
                convert_async_panics(|| async {
                    self.account_manager.delete_storage().await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::GenerateMnemonic => convert_panics(|| {
                self.account_manager
                    .generate_mnemonic()
                    .map(Response::GeneratedMnemonic)
            }),
            Message::VerifyMnemonic(mut mnemonic) => convert_panics(|| {
                self.account_manager.verify_mnemonic(&mnemonic)?;
                mnemonic.zeroize();
                Ok(Response::Ok(()))
            }),
            Message::SetClientOptions(options) => {
                convert_async_panics(|| async {
                    self.account_manager.set_client_options(*options.clone()).await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::GetNodeInfo { url, auth } => {
                convert_async_panics(|| async {
                    match url {
                        Some(url) => {
                            let node_info = Client::get_node_info(&url, auth).await?;
                            Ok(Response::NodeInfo(NodeInfoWrapper { node_info, url }))
                        }
                        None => self.account_manager.get_node_info().await.map(Response::NodeInfo),
                    }
                })
                .await
            }
            Message::SetStrongholdPassword(mut password) => {
                convert_async_panics(|| async {
                    self.account_manager.set_stronghold_password(&password).await?;
                    password.zeroize();
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::SetStrongholdPasswordClearInterval(duration) => {
                convert_async_panics(|| async {
                    self.account_manager
                        .set_stronghold_password_clear_interval(duration)
                        .await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::StoreMnemonic(mnemonic) => {
                convert_async_panics(|| async {
                    self.account_manager.store_mnemonic(mnemonic).await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::StartBackgroundSync { options, interval } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .start_background_syncing(options.clone(), interval)
                        .await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            Message::StopBackgroundSync => {
                convert_async_panics(|| async {
                    self.account_manager.stop_background_syncing()?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            #[cfg(feature = "events")]
            #[cfg(debug_assertions)]
            Message::EmitTestEvent(event) => {
                convert_async_panics(|| async {
                    self.account_manager.emit_test_event(event.clone()).await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => Response::Error(e),
        };

        log::debug!("Response: {:?}", response);

        let _ = response_tx.send(response);
    }

    #[cfg(feature = "stronghold")]
    async fn backup(&self, backup_path: PathBuf, stronghold_password: String) -> Result<Response> {
        self.account_manager.backup(backup_path, stronghold_password).await?;
        Ok(Response::Ok(()))
    }

    #[cfg(feature = "stronghold")]
    async fn restore_backup(&self, backup_path: PathBuf, stronghold_password: String) -> Result<Response> {
        self.account_manager
            .restore_backup(backup_path, stronghold_password)
            .await?;
        Ok(Response::Ok(()))
    }

    async fn call_account_method(&self, account_id: &AccountIdentifier, method: &AccountMethod) -> Result<Response> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;

        match method {
            AccountMethod::GenerateAddresses { amount, options } => {
                let address = account_handle.generate_addresses(*amount, options.clone()).await?;
                Ok(Response::GeneratedAddress(address))
            }
            AccountMethod::GetOutputsWithAdditionalUnlockConditions { outputs_to_collect } => {
                let output_ids = account_handle
                    .get_unlockable_outputs_with_additional_unlock_conditions(*outputs_to_collect)
                    .await?;
                Ok(Response::OutputIds(output_ids))
            }
            AccountMethod::GetOutput { output_id } => {
                let output_data = account_handle.get_output(output_id).await;
                Ok(Response::Output(
                    output_data.as_ref().map(OutputDataDto::from).map(Box::new),
                ))
            }
            AccountMethod::GetTransaction { transaction_id } => {
                let transaction_data = account_handle.get_transaction(transaction_id).await;
                Ok(Response::Transaction(Box::new(transaction_data)))
            }
            AccountMethod::ListAddresses => {
                let addresses = account_handle.list_addresses().await?;
                Ok(Response::Addresses(addresses))
            }
            AccountMethod::ListAddressesWithUnspentOutputs => {
                let addresses = account_handle.list_addresses_with_unspent_outputs().await?;
                Ok(Response::AddressesWithUnspentOutputs(
                    addresses.iter().map(AddressWithUnspentOutputsDto::from).collect(),
                ))
            }
            AccountMethod::ListOutputs => {
                let outputs = account_handle.list_outputs().await?;
                Ok(Response::Outputs(outputs.iter().map(OutputDataDto::from).collect()))
            }
            AccountMethod::ListUnspentOutputs => {
                let outputs = account_handle.list_unspent_outputs().await?;
                Ok(Response::Outputs(outputs.iter().map(OutputDataDto::from).collect()))
            }
            AccountMethod::ListTransactions => {
                let transactions = account_handle.list_transactions().await?;
                Ok(Response::Transactions(
                    transactions.iter().map(TransactionDto::from).collect(),
                ))
            }
            AccountMethod::ListPendingTransactions => {
                let transactions = account_handle.list_pending_transactions().await?;
                Ok(Response::Transactions(
                    transactions.iter().map(TransactionDto::from).collect(),
                ))
            }
            AccountMethod::MintNativeToken {
                native_token_options,
                options,
            } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .mint_native_token(native_token_options.clone(), options.clone())
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::MintNfts { nfts_options, options } => {
                convert_async_panics(|| async {
                    let transfer = account_handle.mint_nfts(nfts_options.clone(), options.clone()).await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::GetBalance => Ok(Response::Balance(AccountBalanceDto::from(
                &account_handle.balance().await?,
            ))),
            AccountMethod::PrepareMintNfts { nfts_options, options } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_mint_nfts(nfts_options.clone(), options.clone())
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::PrepareSendAmount {
                addresses_with_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_send_amount(
                            addresses_with_amount
                                .iter()
                                .map(AddressWithAmount::try_from)
                                .collect::<Result<Vec<AddressWithAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::PrepareSendMicroTransaction {
                addresses_with_micro_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_send_micro_transaction(
                            addresses_with_micro_amount
                                .iter()
                                .map(AddressWithMicroAmount::try_from)
                                .collect::<Result<Vec<AddressWithMicroAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::PrepareSendNativeTokens {
                addresses_native_tokens,
                options,
            } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_send_native_tokens(addresses_native_tokens.clone(), options.clone())
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::PrepareSendNft {
                addresses_nft_ids,
                options,
            } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_send_nft(addresses_nft_ids.clone(), options.clone())
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::PrepareTransaction { outputs, options } => {
                convert_async_panics(|| async {
                    let data = account_handle
                        .prepare_transaction(
                            outputs
                                .iter()
                                .map(|o| Ok(Output::try_from(o)?))
                                .collect::<Result<Vec<Output>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::PreparedTransaction(PreparedTransactionDataDto::from(&data)))
                })
                .await
            }
            AccountMethod::SyncAccount { options } => Ok(Response::Balance(AccountBalanceDto::from(
                &account_handle.sync(options.clone()).await?,
            ))),
            AccountMethod::SendAmount {
                addresses_with_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .send_amount(
                            addresses_with_amount
                                .iter()
                                .map(AddressWithAmount::try_from)
                                .collect::<Result<Vec<AddressWithAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::SendMicroTransaction {
                addresses_with_micro_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .send_micro_transaction(
                            addresses_with_micro_amount
                                .iter()
                                .map(AddressWithMicroAmount::try_from)
                                .collect::<Result<Vec<AddressWithMicroAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::SendNativeTokens {
                addresses_native_tokens,
                options,
            } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .send_native_tokens(addresses_native_tokens.clone(), options.clone())
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::SendNft {
                addresses_nft_ids,
                options,
            } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .send_nft(addresses_nft_ids.clone(), options.clone())
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::SetAlias { alias } => {
                convert_async_panics(|| async {
                    account_handle.set_alias(alias).await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            AccountMethod::SendTransfer { outputs, options } => {
                convert_async_panics(|| async {
                    let transfer = account_handle
                        .send(
                            outputs
                                .iter()
                                .map(|o| Ok(Output::try_from(o)?))
                                .collect::<crate::Result<Vec<Output>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransfer(transfer))
                })
                .await
            }
            AccountMethod::SignTransactionEssence {
                prepared_transaction_data,
            } => {
                convert_async_panics(|| async {
                    let signed_transaction_data = account_handle
                        .sign_transaction_essence(&PreparedTransactionData::try_from(prepared_transaction_data)?)
                        .await?;
                    Ok(Response::SignedTransactionData(SignedTransactionDataDto::from(
                        &signed_transaction_data,
                    )))
                })
                .await
            }
            AccountMethod::SubmitAndStoreTransaction {
                signed_transaction_data,
            } => {
                convert_async_panics(|| async {
                    let signed_transaction_data = SignedTransactionData::try_from(signed_transaction_data)?;
                    let transaction_result = account_handle
                        .submit_and_store_transaction(signed_transaction_data)
                        .await?;
                    Ok(Response::SentTransfer(transaction_result))
                })
                .await
            }
            AccountMethod::TryCollectOutputs { outputs_to_collect } => {
                convert_async_panics(|| async {
                    let transfer_results = account_handle.try_collect_outputs(*outputs_to_collect).await?;
                    Ok(Response::SentTransfers(transfer_results))
                })
                .await
            }
            AccountMethod::CollectOutputs { output_ids_to_collect } => {
                convert_async_panics(|| async {
                    let transfer_results = account_handle.collect_outputs(output_ids_to_collect.to_vec()).await?;
                    Ok(Response::SentTransfers(transfer_results))
                })
                .await
            }
        }
    }

    /// The create account message handler.
    async fn create_account(&self, account: &AccountToCreate) -> Result<Response> {
        let mut builder = self.account_manager.create_account();

        if let Some(alias) = &account.alias {
            builder = builder.with_alias(alias.clone());
        }

        if let Some(coin_type) = &account.coin_type {
            builder = builder.with_coin_type(*coin_type);
        }

        match builder.finish().await {
            Ok(account_handle) => {
                let account = account_handle.read().await;
                Ok(Response::Account(AccountDto::from(&*account)))
            }
            Err(e) => Err(e),
        }
    }

    async fn get_account(&self, account_id: &AccountIdentifier) -> Result<Response> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;
        let account = account_handle.read().await;
        Ok(Response::Account(AccountDto::from(&*account)))
    }

    async fn get_accounts(&self) -> Result<Response> {
        let account_handles = self.account_manager.get_accounts().await?;
        let mut accounts = Vec::new();
        for account_handle in account_handles {
            let account = account_handle.read().await;
            accounts.push(AccountDto::from(&*account));
        }
        Ok(Response::Accounts(accounts))
    }
}

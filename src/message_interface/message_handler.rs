// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    time::Duration,
};

use backtrace::Backtrace;
use futures::{Future, FutureExt};
use iota_client::{
    api::{PreparedTransactionData, PreparedTransactionDataDto, SignedTransactionData, SignedTransactionDataDto},
    block::{
        output::{
            dto::{OutputBuilderAmountDto, OutputDto},
            AliasId, AliasOutput, BasicOutput, FoundryOutput, NftId, NftOutput, Output, Rent, TokenId,
        },
        payload::transaction::dto::TransactionPayloadDto,
        DtoError,
    },
    constants::SHIMMER_TESTNET_BECH32_HRP,
    utils, Client, NodeInfoWrapper,
};
use primitive_types::U256;
use tokio::sync::mpsc::UnboundedSender;
use zeroize::Zeroize;

#[cfg(feature = "events")]
use crate::events::types::{Event, WalletEventType};
use crate::{
    account::{
        operations::transaction::{
            high_level::{create_alias::AliasOutputOptions, minting::mint_native_token::MintTokenTransactionDto},
            prepare_output::OutputOptions,
        },
        types::{AccountIdentifier, TransactionDto},
        OutputDataDto,
    },
    account_manager::AccountManager,
    message_interface::{
        account_method::AccountMethod,
        dtos::{AccountBalanceDto, AccountDto},
        message::Message,
        response::Response,
        AddressWithUnspentOutputsDto,
    },
    AddressWithAmount, AddressWithMicroAmount, IncreaseNativeTokenSupplyOptions, NativeTokenOptions, NftOptions,
    Result,
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

    #[cfg(feature = "events")]
    /// Remove wallet event listeners, empty vec will remove all listeners
    pub async fn clear_listeners(&self, events: Vec<WalletEventType>) {
        self.account_manager.clear_listeners(events).await;
    }

    /// Handles a message.
    pub async fn handle(&self, message: Message, response_tx: UnboundedSender<Response>) {
        log::debug!("Message: {:?}", message);

        let response: Result<Response> = match message {
            Message::CreateAccount { alias } => {
                convert_async_panics(|| async { self.create_account(alias).await }).await
            }
            Message::GetAccount(account_id) => {
                convert_async_panics(|| async { self.get_account(&account_id).await }).await
            }
            Message::GetAccountIndexes => {
                convert_async_panics(|| async {
                    let accounts = self.account_manager.get_accounts().await?;
                    let mut account_indexes = Vec::new();
                    for account in accounts.iter() {
                        account_indexes.push(*account.read().await.index());
                    }
                    Ok(Response::AccountIndexes(account_indexes))
                })
                .await
            }
            Message::GetAccounts => convert_async_panics(|| async { self.get_accounts().await }).await,
            Message::CallAccountMethod { account_id, method } => {
                convert_async_panics(|| async { self.call_account_method(&account_id, method).await }).await
            }
            #[cfg(feature = "stronghold")]
            Message::Backup { destination, password } => {
                convert_async_panics(|| async { self.backup(destination.to_path_buf(), password).await }).await
            }
            #[cfg(feature = "stronghold")]
            Message::ChangeStrongholdPassword {
                mut current_password,
                mut new_password,
            } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .change_stronghold_password(&current_password, &new_password)
                        .await?;
                    current_password.zeroize();
                    new_password.zeroize();
                    Ok(Response::Ok(()))
                })
                .await
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
                account_start_index,
                account_gap_limit,
                address_gap_limit,
                sync_options,
            } => {
                convert_async_panics(|| async {
                    let account_handles = self
                        .account_manager
                        .recover_accounts(account_start_index, account_gap_limit, address_gap_limit, sync_options)
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
            Message::RemoveLatestAccount => {
                convert_async_panics(|| async {
                    self.account_manager.remove_latest_account().await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            #[cfg(feature = "stronghold")]
            Message::RestoreBackup { source, password } => {
                convert_async_panics(|| async { self.restore_backup(source.to_path_buf(), password).await }).await
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
            #[cfg(feature = "ledger_nano")]
            Message::GetLedgerNanoStatus => {
                convert_async_panics(|| async {
                    let ledger_nano_status = self.account_manager.get_ledger_nano_status().await?;
                    Ok(Response::LedgerNanoStatus(ledger_nano_status))
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
            Message::SetStrongholdPasswordClearInterval(interval_in_milliseconds) => {
                convert_async_panics(|| async {
                    let duration = interval_in_milliseconds.map(Duration::from_millis);
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
            Message::StartBackgroundSync {
                options,
                interval_in_milliseconds,
            } => {
                convert_async_panics(|| async {
                    let duration = interval_in_milliseconds.map(Duration::from_millis);
                    self.account_manager
                        .start_background_syncing(options.clone(), duration)
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
            Message::Bech32ToHex(bech32) => convert_panics(|| Ok(Response::HexAddress(utils::bech32_to_hex(&bech32)?))),
            Message::HexToBech32 { hex, bech32_hrp } => {
                convert_async_panics(|| async {
                    let bech32_hrp = match bech32_hrp {
                        Some(bech32_hrp) => bech32_hrp,
                        None => match self.account_manager.get_node_info().await {
                            Ok(node_info_wrapper) => node_info_wrapper.node_info.protocol.bech32_hrp,
                            Err(_) => SHIMMER_TESTNET_BECH32_HRP.into(),
                        },
                    };

                    Ok(Response::Bech32Address(utils::hex_to_bech32(&hex, &bech32_hrp)?))
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

    async fn call_account_method(&self, account_id: &AccountIdentifier, method: AccountMethod) -> Result<Response> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;

        match method {
            AccountMethod::BuildAliasOutput {
                amount,
                native_tokens,
                alias_id,
                state_index,
                state_metadata,
                foundry_counter,
                unlock_conditions,
                features,
                immutable_features,
            } => {
                let output = Output::from(AliasOutput::try_from_dtos(
                    if let Some(amount) = amount {
                        OutputBuilderAmountDto::Amount(amount)
                    } else {
                        OutputBuilderAmountDto::MinimumStorageDeposit(account_handle.client.get_rent_structure()?)
                    },
                    native_tokens,
                    &alias_id,
                    state_index,
                    state_metadata,
                    foundry_counter,
                    unlock_conditions,
                    features,
                    immutable_features,
                    account_handle.client.get_token_supply()?,
                )?);

                Ok(Response::Output(OutputDto::from(&output)))
            }
            AccountMethod::BuildBasicOutput {
                amount,
                native_tokens,
                unlock_conditions,
                features,
            } => {
                let output = Output::from(BasicOutput::try_from_dtos(
                    if let Some(amount) = amount {
                        OutputBuilderAmountDto::Amount(amount)
                    } else {
                        OutputBuilderAmountDto::MinimumStorageDeposit(account_handle.client.get_rent_structure()?)
                    },
                    native_tokens,
                    unlock_conditions,
                    features,
                    account_handle.client.get_token_supply()?,
                )?);

                Ok(Response::Output(OutputDto::from(&output)))
            }
            AccountMethod::BuildFoundryOutput {
                amount,
                native_tokens,
                serial_number,
                token_scheme,
                unlock_conditions,
                features,
                immutable_features,
            } => {
                let output = Output::from(FoundryOutput::try_from_dtos(
                    if let Some(amount) = amount {
                        OutputBuilderAmountDto::Amount(amount)
                    } else {
                        OutputBuilderAmountDto::MinimumStorageDeposit(account_handle.client.get_rent_structure()?)
                    },
                    native_tokens,
                    serial_number,
                    &token_scheme,
                    unlock_conditions,
                    features,
                    immutable_features,
                    account_handle.client.get_token_supply()?,
                )?);

                Ok(Response::Output(OutputDto::from(&output)))
            }
            AccountMethod::BuildNftOutput {
                amount,
                native_tokens,
                nft_id,
                unlock_conditions,
                features,
                immutable_features,
            } => {
                let output = Output::from(NftOutput::try_from_dtos(
                    if let Some(amount) = amount {
                        OutputBuilderAmountDto::Amount(amount)
                    } else {
                        OutputBuilderAmountDto::MinimumStorageDeposit(account_handle.client.get_rent_structure()?)
                    },
                    native_tokens,
                    &nft_id,
                    unlock_conditions,
                    features,
                    immutable_features,
                    account_handle.client.get_token_supply()?,
                )?);

                Ok(Response::Output(OutputDto::from(&output)))
            }
            AccountMethod::BurnNativeToken {
                token_id,
                burn_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .burn_native_token(
                            TokenId::try_from(&token_id)?,
                            U256::try_from(&burn_amount).map_err(|_| DtoError::InvalidField("burn_amount"))?,
                            options,
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::BurnNft { nft_id, options } => {
                convert_async_panics(|| async {
                    let transaction = account_handle.burn_nft(NftId::try_from(&nft_id)?, options).await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::ConsolidateOutputs {
                force,
                output_consolidation_threshold,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .consolidate_outputs(force, output_consolidation_threshold)
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::CreateAliasOutput {
                alias_output_options,
                options,
            } => {
                convert_async_panics(|| async {
                    let alias_output_options = alias_output_options
                        .map(|options| AliasOutputOptions::try_from(&options))
                        .transpose()?;

                    let transaction = account_handle
                        .create_alias_output(alias_output_options, options)
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::DestroyAlias { alias_id, options } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .destroy_alias(AliasId::try_from(&alias_id)?, options)
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::DestroyFoundry { foundry_id, options } => {
                convert_async_panics(|| async {
                    let transaction = account_handle.destroy_foundry(foundry_id, options).await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::GenerateAddresses { amount, options } => {
                let address = account_handle.generate_addresses(amount, options.clone()).await?;
                Ok(Response::GeneratedAddress(address))
            }
            AccountMethod::GetOutputsWithAdditionalUnlockConditions { outputs_to_claim } => {
                let output_ids = account_handle
                    .get_unlockable_outputs_with_additional_unlock_conditions(outputs_to_claim)
                    .await?;
                Ok(Response::OutputIds(output_ids))
            }
            AccountMethod::GetOutput { output_id } => {
                let output_data = account_handle.get_output(&output_id).await;
                Ok(Response::OutputData(
                    output_data.as_ref().map(OutputDataDto::from).map(Box::new),
                ))
            }
            AccountMethod::GetFoundryOutput { token_id } => {
                let token_id = TokenId::try_from(&token_id)?;
                let output = account_handle.get_foundry_output(token_id).await?;
                Ok(Response::Output(OutputDto::from(&output)))
            }
            AccountMethod::GetTransaction { transaction_id } => {
                let transaction = account_handle.get_transaction(&transaction_id).await;
                Ok(Response::Transaction(
                    transaction.as_ref().map(TransactionDto::from).map(Box::new),
                ))
            }
            AccountMethod::GetIncomingTransactionData { transaction_id } => {
                let transaction_data = account_handle.get_incoming_transaction_data(&transaction_id).await;
                match transaction_data {
                    Some((transaction_payload, inputs)) => Ok(Response::IncomingTransactionData(Some(Box::new((
                        transaction_id,
                        (TransactionPayloadDto::from(&transaction_payload), inputs),
                    ))))),
                    None => Ok(Response::IncomingTransactionData(None)),
                }
            }
            AccountMethod::Addresses => {
                let addresses = account_handle.addresses().await?;
                Ok(Response::Addresses(addresses))
            }
            AccountMethod::AddressesWithUnspentOutputs => {
                let addresses = account_handle.addresses_with_unspent_outputs().await?;
                Ok(Response::AddressesWithUnspentOutputs(
                    addresses.iter().map(AddressWithUnspentOutputsDto::from).collect(),
                ))
            }
            AccountMethod::Outputs { filter_options } => {
                let outputs = account_handle.outputs(filter_options).await?;
                Ok(Response::OutputsData(outputs.iter().map(OutputDataDto::from).collect()))
            }
            AccountMethod::UnspentOutputs { filter_options } => {
                let outputs = account_handle.unspent_outputs(filter_options).await?;
                Ok(Response::OutputsData(outputs.iter().map(OutputDataDto::from).collect()))
            }
            AccountMethod::IncomingTransactions => {
                let transactions = account_handle.incoming_transactions().await?;
                Ok(Response::IncomingTransactionsData(
                    transactions
                        .into_iter()
                        .map(|d| (d.0, (TransactionPayloadDto::from(&d.1.0), d.1.1)))
                        .collect(),
                ))
            }
            AccountMethod::Transactions => {
                let transactions = account_handle.transactions().await?;
                Ok(Response::Transactions(
                    transactions.iter().map(TransactionDto::from).collect(),
                ))
            }
            AccountMethod::PendingTransactions => {
                let transactions = account_handle.pending_transactions().await?;
                Ok(Response::Transactions(
                    transactions.iter().map(TransactionDto::from).collect(),
                ))
            }
            AccountMethod::DecreaseNativeTokenSupply {
                token_id,
                melt_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .decrease_native_token_supply(
                            TokenId::try_from(&token_id)?,
                            U256::try_from(&melt_amount).map_err(|_| DtoError::InvalidField("melt_amount"))?,
                            options,
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::IncreaseNativeTokenSupply {
                token_id,
                mint_amount,
                increase_native_token_supply_options,
                options,
            } => {
                convert_async_panics(|| async {
                    let increase_native_token_supply_options = match increase_native_token_supply_options {
                        Some(native_token_options) => {
                            Some(IncreaseNativeTokenSupplyOptions::try_from(&native_token_options)?)
                        }
                        None => None,
                    };
                    let transaction = account_handle
                        .increase_native_token_supply(
                            TokenId::try_from(&token_id)?,
                            U256::try_from(&mint_amount).map_err(|_| DtoError::InvalidField("mint_amount"))?,
                            increase_native_token_supply_options,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::MintTokenTransaction(MintTokenTransactionDto::from(
                        &transaction,
                    )))
                })
                .await
            }
            AccountMethod::MintNativeToken {
                native_token_options,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .mint_native_token(NativeTokenOptions::try_from(&native_token_options)?, options.clone())
                        .await?;
                    Ok(Response::MintTokenTransaction(MintTokenTransactionDto::from(
                        &transaction,
                    )))
                })
                .await
            }
            AccountMethod::MinimumRequiredStorageDeposit { output } => {
                convert_async_panics(|| async {
                    let output = Output::try_from_dto(&output, account_handle.client.get_token_supply()?)?;
                    let rent_structure = account_handle.client.get_rent_structure()?;

                    let minimum_storage_deposit = output.rent_cost(&rent_structure);

                    Ok(Response::MinimumRequiredStorageDeposit(
                        minimum_storage_deposit.to_string(),
                    ))
                })
                .await
            }
            AccountMethod::MintNfts { nfts_options, options } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .mint_nfts(
                            nfts_options
                                .iter()
                                .map(NftOptions::try_from)
                                .collect::<Result<Vec<NftOptions>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::GetBalance => Ok(Response::Balance(AccountBalanceDto::from(
                &account_handle.balance().await?,
            ))),
            AccountMethod::PrepareOutput {
                options,
                transaction_options,
            } => {
                convert_async_panics(|| async {
                    let output = account_handle
                        .prepare_output(OutputOptions::try_from(&options)?, transaction_options.clone())
                        .await?;
                    Ok(Response::Output(OutputDto::from(&output)))
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
            AccountMethod::PrepareTransaction { outputs, options } => {
                convert_async_panics(|| async {
                    let token_supply = account_handle.client.get_token_supply()?;
                    let data = account_handle
                        .prepare_transaction(
                            outputs
                                .iter()
                                .map(|o| Ok(Output::try_from_dto(o, token_supply)?))
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
                    let transaction = account_handle
                        .send_amount(
                            addresses_with_amount
                                .iter()
                                .map(AddressWithAmount::try_from)
                                .collect::<Result<Vec<AddressWithAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::SendMicroTransaction {
                addresses_with_micro_amount,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .send_micro_transaction(
                            addresses_with_micro_amount
                                .iter()
                                .map(AddressWithMicroAmount::try_from)
                                .collect::<Result<Vec<AddressWithMicroAmount>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::SendNativeTokens {
                addresses_native_tokens,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .send_native_tokens(addresses_native_tokens.clone(), options.clone())
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::SendNft {
                addresses_nft_ids,
                options,
            } => {
                convert_async_panics(|| async {
                    let transaction = account_handle
                        .send_nft(addresses_nft_ids.clone(), options.clone())
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::SetAlias { alias } => {
                convert_async_panics(|| async {
                    account_handle.set_alias(&alias).await?;
                    Ok(Response::Ok(()))
                })
                .await
            }
            AccountMethod::SendOutputs { outputs, options } => {
                convert_async_panics(|| async {
                    let token_supply = account_handle.client.get_token_supply()?;
                    let transaction = account_handle
                        .send(
                            outputs
                                .iter()
                                .map(|o| Ok(Output::try_from_dto(o, token_supply)?))
                                .collect::<crate::Result<Vec<Output>>>()?,
                            options.clone(),
                        )
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::SignTransactionEssence {
                prepared_transaction_data,
            } => {
                convert_async_panics(|| async {
                    let signed_transaction_data = account_handle
                        .sign_transaction_essence(&PreparedTransactionData::try_from_dto(
                            &prepared_transaction_data,
                            &account_handle.client.get_protocol_parameters()?,
                        )?)
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
                    let signed_transaction_data = SignedTransactionData::try_from_dto(
                        &signed_transaction_data,
                        &account_handle.client.get_protocol_parameters()?,
                    )?;
                    let transaction = account_handle
                        .submit_and_store_transaction(signed_transaction_data)
                        .await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
            AccountMethod::ClaimOutputs { output_ids_to_claim } => {
                convert_async_panics(|| async {
                    let transaction = account_handle.claim_outputs(output_ids_to_claim.to_vec()).await?;
                    Ok(Response::SentTransaction(TransactionDto::from(&transaction)))
                })
                .await
            }
        }
    }

    /// The create account message handler.
    async fn create_account(&self, alias: Option<String>) -> Result<Response> {
        let mut builder = self.account_manager.create_account();

        if let Some(alias) = alias {
            builder = builder.with_alias(alias);
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

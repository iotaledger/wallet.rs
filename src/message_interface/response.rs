// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Formatter, Result};

#[cfg(feature = "ledger_nano")]
use iota_client::secret::LedgerNanoStatus;
use iota_client::{
    api::{PreparedTransactionDataDto, SignedTransactionDataDto},
    api_types::response::OutputResponse,
    block::{
        output::{dto::OutputDto, OutputId},
        payload::transaction::{dto::TransactionPayloadDto, TransactionId},
    },
    NodeInfoWrapper,
};
use serde::Serialize;

use crate::{
    account::{
        operations::transaction::high_level::minting::mint_native_token::MintTokenTransactionDto,
        types::{address::AccountAddress, TransactionDto},
        OutputDataDto,
    },
    message_interface::dtos::{AccountBalanceDto, AccountDto, AddressWithUnspentOutputsDto},
    Error,
};

type IncomingTransactionDataDto = (TransactionPayloadDto, Vec<OutputResponse>);

/// The response message.
#[derive(Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Response {
    /// Response for
    /// [`CreateAccount`](crate::message_interface::Message::CreateAccount),
    /// [`GetAccount`](crate::message_interface::Message::GetAccount)
    Account(AccountDto),
    /// Response for [`GetAccountIndexes`](crate::message_interface::Message::GetAccountIndexes)
    AccountIndexes(Vec<u32>),
    /// Response for [`GetAccounts`](crate::message_interface::Message::GetAccounts)
    Accounts(Vec<AccountDto>),
    /// Response for [`Addresses`](crate::message_interface::AccountMethod::Addresses)
    Addresses(Vec<AccountAddress>),
    /// Response for
    /// [`AddressesWithUnspentOutputs`](crate::message_interface::AccountMethod::AddressesWithUnspentOutputs)
    AddressesWithUnspentOutputs(Vec<AddressWithUnspentOutputsDto>),
    /// Response for
    /// [`BuildAliasOutput`](crate::message_interface::AccountMethod::BuildAliasOutput)
    /// [`BuildBasicOutput`](crate::message_interface::AccountMethod::BuildBasicOutput)
    /// [`BuildFoundryOutput`](crate::message_interface::AccountMethod::BuildFoundryOutput)
    /// [`BuildNftOutput`](crate::message_interface::AccountMethod::BuildNftOutput)
    /// [`GetFoundryOutput`](crate::message_interface::AccountMethod::GetFoundryOutput)
    /// [`PrepareOutput`](crate::message_interface::AccountMethod::PrepareOutput)
    Output(OutputDto),
    /// Response for
    /// [`MinimumRequiredStorageDeposit`](crate::message_interface::AccountMethod::MinimumRequiredStorageDeposit)
    MinimumRequiredStorageDeposit(String),
    /// Response for
    /// [`GetOutputsWithAdditionalUnlockConditions`](crate::message_interface::AccountMethod::
    /// GetOutputsWithAdditionalUnlockConditions)
    OutputIds(Vec<OutputId>),
    /// Response for [`GetOutput`](crate::message_interface::AccountMethod::GetOutput)
    OutputData(Option<Box<OutputDataDto>>),
    /// Response for
    /// [`Outputs`](crate::message_interface::AccountMethod::Outputs),
    /// [`UnspentOutputs`](crate::message_interface::AccountMethod::UnspentOutputs)
    OutputsData(Vec<OutputDataDto>),
    /// Response for
    /// [`PrepareSendAmount`](crate::message_interface::AccountMethod::PrepareSendAmount),
    /// [`PrepareTransaction`](crate::message_interface::AccountMethod::PrepareTransaction)
    PreparedTransaction(PreparedTransactionDataDto),
    /// Response for
    /// [`GetTransaction`](crate::message_interface::AccountMethod::GetTransaction),
    Transaction(Option<Box<TransactionDto>>),
    /// Response for
    /// [`Transactions`](crate::message_interface::AccountMethod::Transactions),
    /// [`PendingTransactions`](crate::message_interface::AccountMethod::PendingTransactions)
    Transactions(Vec<TransactionDto>),
    /// Response for
    /// [`SignTransaction`](crate::message_interface::AccountMethod::SignTransaction)
    SignedTransactionData(SignedTransactionDataDto),
    /// GenerateAddress response.
    /// Response for [`GenerateAddresses`](crate::message_interface::AccountMethod::GenerateAddresses)
    GeneratedAddress(Vec<AccountAddress>),
    /// Response for
    /// [`GetBalance`](crate::message_interface::AccountMethod::GetBalance),
    /// [`SyncAccount`](crate::message_interface::AccountMethod::SyncAccount)
    Balance(AccountBalanceDto),
    /// Response for
    /// [`GetLedgerNanoStatus`](crate::message_interface::Message::GetLedgerNanoStatus),
    #[cfg(feature = "ledger_nano")]
    LedgerNanoStatus(LedgerNanoStatus),
    /// Response for
    /// [`GetIncomingTransactionData`](crate::message_interface::AccountMethod::GetIncomingTransactionData),
    IncomingTransactionData(Option<Box<(TransactionId, IncomingTransactionDataDto)>>),
    /// Response for
    /// [`IncomingTransactions`](crate::message_interface::AccountMethod::IncomingTransactions),
    IncomingTransactionsData(Vec<(TransactionId, IncomingTransactionDataDto)>),
    /// Response for
    /// [`ConsolidateOutputs`](crate::message_interface::AccountMethod::ConsolidateOutputs)
    /// [`ClaimOutputs`](crate::message_interface::AccountMethod::ClaimOutputs)
    /// [`CreateAliasOutput`](crate::message_interface::AccountMethod::CreateAliasOutput)
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`MintNfts`](crate::message_interface::AccountMethod::MintNfts),
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`SendMicroTransaction`](crate::message_interface::AccountMethod::SendMicroTransaction),
    /// [`SendNativeTokens`](crate::message_interface::AccountMethod::SendNativeTokens),
    /// [`SendNft`](crate::message_interface::AccountMethod::SendNft),
    /// [`SendOutputs`](crate::message_interface::AccountMethod::SendOutputs)
    /// [`SubmitAndStoreTransaction`](crate::message_interface::AccountMethod::SubmitAndStoreTransaction)
    SentTransaction(TransactionDto),
    /// Response for
    /// [`MintNativeToken`](crate::message_interface::AccountMethod::MintNativeToken),
    MintTokenTransaction(MintTokenTransactionDto),
    /// Response for
    /// [`IsStrongholdPasswordAvailable`](crate::message_interface::Message::IsStrongholdPasswordAvailable)
    StrongholdPasswordIsAvailable(bool),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// Response for [`GenerateMnemonic`](crate::message_interface::Message::GenerateMnemonic)
    GeneratedMnemonic(String),
    /// Response for [`GetNodeInfo`](crate::message_interface::Message::GetNodeInfo)
    NodeInfo(NodeInfoWrapper),
    /// Response for [`Bech32ToHex`](crate::message_interface::Message::Bech32ToHex)
    HexAddress(String),
    /// Response for [`HexToBech32`](crate::message_interface::Message::HexToBech32)
    Bech32Address(String),
    /// Response for
    /// [`Backup`](crate::message_interface::Message::Backup),
    /// [`ClearStrongholdPassword`](crate::message_interface::Message::ClearStrongholdPassword),
    /// [`RestoreBackup`](crate::message_interface::Message::RestoreBackup),
    /// [`VerifyMnemonic`](crate::message_interface::Message::VerifyMnemonic),
    /// [`SetClientOptions`](crate::message_interface::Message::SetClientOptions),
    /// [`SetStrongholdPassword`](crate::message_interface::Message::SetStrongholdPassword),
    /// [`SetStrongholdPasswordClearInterval`](crate::message_interface::Message::
    /// SetStrongholdPasswordClearInterval),
    /// [`StoreMnemonic`](crate::message_interface::Message::StoreMnemonic),
    /// [`StartBackgroundSync`](crate::message_interface::Message::StartBackgroundSync),
    /// [`StopBackgroundSync`](crate::message_interface::Message::StopBackgroundSync),
    /// [`EmitTestEvent`](crate::message_interface::Message::EmitTestEvent),
    Ok(()),
}

// Custom Debug implementation to not log secrets
impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Response::Account(account) => write!(f, "Account({:?})", account),
            Response::AccountIndexes(account_indexes) => write!(f, "AccountIndexes({:?})", account_indexes),
            Response::Accounts(accounts) => write!(f, "Accounts({:?})", accounts),
            Response::Addresses(addresses) => write!(f, "Addresses({:?})", addresses),
            Response::AddressesWithUnspentOutputs(addresses) => {
                write!(f, "AddressesWithUnspentOutputs({:?})", addresses)
            }
            Response::Output(output) => write!(f, "Output({:?})", output),
            Response::MinimumRequiredStorageDeposit(amount) => write!(f, "MinimumRequiredStorageDeposit({:?})", amount),
            Response::OutputIds(output_ids) => write!(f, "OutputIds({:?})", output_ids),
            Response::OutputData(output) => write!(f, "OutputData({:?})", output),
            Response::OutputsData(outputs) => write!(f, "OutputsData{:?}", outputs),
            Response::PreparedTransaction(transaction_data) => {
                write!(f, "PreparedTransaction({:?})", transaction_data)
            }
            Response::Transaction(transaction) => write!(f, "Transaction({:?})", transaction),
            Response::Transactions(transactions) => write!(f, "Transactions({:?})", transactions),
            Response::SignedTransactionData(signed_transaction_data) => {
                write!(f, "SignedTransactionData({:?})", signed_transaction_data)
            }
            Response::GeneratedAddress(addresses) => write!(f, "GeneratedAddress({:?})", addresses),
            Response::Balance(balance) => write!(f, "Balance({:?})", balance),
            Response::IncomingTransactionData(transaction_data) => {
                write!(f, "IncomingTransactionData({:?})", transaction_data)
            }
            Response::IncomingTransactionsData(transactions_data) => {
                write!(f, "IncomingTransactionsData({:?})", transactions_data)
            }
            Response::SentTransaction(transaction) => write!(f, "SentTransaction({:?})", transaction),
            Response::MintTokenTransaction(mint_transaction) => {
                write!(f, "MintTokenTransaction({:?})", mint_transaction)
            }
            Response::StrongholdPasswordIsAvailable(is_available) => {
                write!(f, "StrongholdPasswordIsAvailable({:?})", is_available)
            }
            Response::Error(error) => write!(f, "Error({:?})", error),
            Response::Panic(panic_msg) => write!(f, "Panic({:?})", panic_msg),
            Response::GeneratedMnemonic(_) => write!(f, "GeneratedMnemonic(<omitted>)"),
            #[cfg(feature = "ledger_nano")]
            Response::LedgerNanoStatus(ledger_nano_status) => write!(f, "LedgerNanoStatus({:?})", ledger_nano_status),
            Response::NodeInfo(info) => write!(f, "NodeInfo({:?})", info),
            Response::HexAddress(hex_address) => write!(f, "Hex encoded address({:?})", hex_address),
            Response::Bech32Address(bech32_address) => write!(f, "Bech32 encoded address({:?})", bech32_address),
            Response::Ok(()) => write!(f, "Ok(())"),
        }
    }
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Formatter, Result};

#[cfg(feature = "ledger_nano")]
use iota_client::secret::LedgerStatus;
use iota_client::{
    api::{PreparedTransactionDataDto, SignedTransactionDataDto},
    bee_block::{
        output::{dto::OutputDto, OutputId},
        payload::transaction::{dto::TransactionPayloadDto, TransactionId},
    },
    bee_rest_api::types::responses::OutputResponse,
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
#[serde(tag = "type", content = "payload")]
pub enum Response {
    /// Response for
    /// [`CreateAccount`](crate::message_interface::Message::CreateAccount),
    /// [`GetAccount`](crate::message_interface::Message::GetAccount)
    Account(AccountDto),
    /// Response for [`GetAccounts`](crate::message_interface::Message::GetAccounts)
    Accounts(Vec<AccountDto>),
    /// Response for [`ListAddresses`](crate::message_interface::AccountMethod::ListAddresses)
    Addresses(Vec<AccountAddress>),
    /// Response for
    /// [`ListAddressesWithUnspentOutputs`](crate::message_interface::AccountMethod::ListAddressesWithUnspentOutputs)
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
    /// [`ListOutputs`](crate::message_interface::AccountMethod::ListOutputs),
    /// [`ListUnspentOutputs`](crate::message_interface::AccountMethod::ListUnspentOutputs)
    OutputsData(Vec<OutputDataDto>),
    /// Response for
    /// [`PrepareSendAmount`](crate::message_interface::AccountMethod::PrepareSendAmount),
    /// [`PrepareTransaction`](crate::message_interface::AccountMethod::PrepareTransaction)
    PreparedTransaction(PreparedTransactionDataDto),
    /// Response for
    /// [`GetTransaction`](crate::message_interface::AccountMethod::GetTransaction),
    Transaction(Option<Box<TransactionDto>>),
    /// Response for
    /// [`ListTransactions`](crate::message_interface::AccountMethod::ListTransactions),
    /// [`ListPendingTransactions`](crate::message_interface::AccountMethod::ListPendingTransactions)
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
    /// [`GetLedgerStatus`](crate::message_interface::Message::GetLedgerStatus),
    #[cfg(feature = "ledger_nano")]
    LedgerStatus(LedgerStatus),
    /// Response for
    /// [`GetIncomingTransactionData`](crate::message_interface::AccountMethod::GetIncomingTransactionData),
    IncomingTransactionData(Option<Box<(TransactionId, IncomingTransactionDataDto)>>),
    /// Response for
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`MintNfts`](crate::message_interface::AccountMethod::MintNfts),
    /// [`SendMicroTransaction`](crate::message_interface::AccountMethod::SendMicroTransaction),
    /// [`SendNativeTokens`](crate::message_interface::AccountMethod::SendNativeTokens),
    /// [`SendNft`](crate::message_interface::AccountMethod::SendNft),
    /// [`SendOutputs`](crate::message_interface::AccountMethod::SendOutputs)
    /// [`SubmitAndStoreTransaction`](crate::message_interface::AccountMethod::SubmitAndStoreTransaction)
    SentTransaction(TransactionDto),
    /// Response for
    /// [`TryClaimOutputs`](crate::message_interface::AccountMethod::TryClaimOutputs),
    /// [`ClaimOutputs`](crate::message_interface::AccountMethod::ClaimOutputs)
    /// [`ConsolidateOutputs`](crate::message_interface::AccountMethod::ConsolidateOutputs)
    SentTransactions(Vec<TransactionDto>),
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
    /// [`DeleteAccountsAndDatabase`](crate::message_interface::Message::DeleteAccountsAndDatabase),
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
            Response::SentTransaction(transaction) => write!(f, "SentTransaction({:?})", transaction),
            Response::SentTransactions(transactions) => write!(f, "SentTransactions({:?})", transactions),
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
            Response::LedgerStatus(ledger_status) => write!(f, "LedgerStatus({:?})", ledger_status),
            Response::NodeInfo(info) => write!(f, "NodeInfo({:?})", info),
            Response::HexAddress(hex_address) => write!(f, "Hex encoded address({:?})", hex_address),
            Response::Bech32Address(bech32_address) => write!(f, "Bech32 encoded address({:?})", bech32_address),
            Response::Ok(()) => write!(f, "Ok(())"),
        }
    }
}

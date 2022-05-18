// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Formatter, Result};

use iota_client::{bee_message::output::OutputId, NodeInfoWrapper};
use serde::Serialize;

use crate::{
    account::{
        operations::transfer::TransferResult,
        types::{address::AccountAddress, OutputData, Transaction},
    },
    message_interface::dtos::{AccountBalanceDto, AccountDto, AddressWithUnspentOutputsDto},
    Error,
};

/// The response message.
#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    /// Response for
    /// [`CreateAccount`](crate::message_interface::MessageType::CreateAccount),
    /// [`GetAccount`](crate::message_interface::MessageType::GetAccount)
    Account(AccountDto),
    /// Response for [`GetAccounts`](crate::message_interface::MessageType::GetAccounts)
    Accounts(Vec<AccountDto>),
    /// Response for [`ListAddresses`](crate::message_interface::AccountMethod::ListAddresses)
    Addresses(Vec<AccountAddress>),
    /// Response for
    /// [`ListAddressesWithUnspentOutputs`](crate::message_interface::AccountMethod::ListAddressesWithUnspentOutputs)
    AddressesWithUnspentOutputs(Vec<AddressWithUnspentOutputsDto>),
    /// Response for
    /// [`GetOutputsWithAdditionalUnlockConditions`](crate::message_interface::AccountMethod::
    /// GetOutputsWithAdditionalUnlockConditions)
    OutputIds(Vec<OutputId>),
    /// Response for [`GetOutput`](crate::message_interface::AccountMethod::GetOutput)
    Output(Box<Option<OutputData>>),
    /// Response for
    /// [`ListOutputs`](crate::message_interface::AccountMethod::ListOutputs),
    /// [`ListUnspentOutputs`](crate::message_interface::AccountMethod::ListUnspentOutputs)
    Outputs(Vec<OutputData>),
    /// Response for
    /// [`ListTransactions`](crate::message_interface::AccountMethod::ListTransactions),
    /// [`ListPendingTransactions`](crate::message_interface::AccountMethod::ListPendingTransactions)
    Transactions(Vec<Transaction>),
    /// Response for [`GenerateAddresses`](crate::message_interface::AccountMethod::GenerateAddresses)
    GeneratedAddress(Vec<AccountAddress>),
    /// Response for
    /// [`GetBalance`](crate::message_interface::AccountMethod::GetBalance),
    /// [`SyncAccount`](crate::message_interface::AccountMethod::SyncAccount)
    Balance(AccountBalanceDto),
    /// Response for
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`MintNativeToken`](crate::message_interface::AccountMethod::MintNativeToken),
    /// [`MintNfts`](crate::message_interface::AccountMethod::MintNfts),
    /// [`SendMicroTransaction`](crate::message_interface::AccountMethod::SendMicroTransaction),
    /// [`SendNativeTokens`](crate::message_interface::AccountMethod::SendNativeTokens),
    /// [`SendNft`](crate::message_interface::AccountMethod::SendNft),
    /// [`SendTransfer`](crate::message_interface::AccountMethod::SendTransfer)
    SentTransfer(TransferResult),
    /// Response for [`TryCollectOutputs`](crate::message_interface::AccountMethod::TryCollectOutputs),
    /// [`CollectOutputs`](crate::message_interface::AccountMethod::CollectOutputs)
    SentTransfers(Vec<TransferResult>),
    /// Response for
    /// [`IsStrongholdPasswordAvailable`](crate::message_interface::MessageType::IsStrongholdPasswordAvailable)
    StrongholdPasswordIsAvailable(bool),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// Response for [`GenerateMnemonic`](crate::message_interface::MessageType::GenerateMnemonic)
    GeneratedMnemonic(String),
    /// Response for [`GetNodeInfo`](crate::message_interface::MessageType::GetNodeInfo)
    NodeInfo(NodeInfoWrapper),
    /// Response for
    /// [`Backup`](crate::message_interface::MessageType::Backup),
    /// [`ClearStrongholdPassword`](crate::message_interface::MessageType::ClearStrongholdPassword),
    /// [`RestoreBackup`](crate::message_interface::MessageType::RestoreBackup),
    /// [`DeleteStorage`](crate::message_interface::MessageType::DeleteStorage),
    /// [`VerifyMnemonic`](crate::message_interface::MessageType::VerifyMnemonic),
    /// [`SetClientOptions`](crate::message_interface::MessageType::SetClientOptions),
    /// [`SetStrongholdPassword`](crate::message_interface::MessageType::SetStrongholdPassword),
    /// [`StoreMnemonic`](crate::message_interface::MessageType::StoreMnemonic),
    /// [`StartBackgroundSync`](crate::message_interface::MessageType::StartBackgroundSync),
    /// [`StopBackgroundSync`](crate::message_interface::MessageType::StopBackgroundSync),
    /// [`EmitTestEvent`](crate::message_interface::MessageType::EmitTestEvent),
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
            Response::OutputIds(output_ids) => write!(f, "OutputIds({:?})", output_ids),
            Response::Output(output) => write!(f, "Output({:?})", output),
            Response::Outputs(outputs) => write!(f, "Outputs{:?}", outputs),
            Response::Transactions(transactions) => write!(f, "Transactions({:?})", transactions),
            Response::GeneratedAddress(addresses) => write!(f, "GeneratedAddress({:?})", addresses),
            Response::Balance(balance) => write!(f, "Balance({:?})", balance),
            Response::SentTransfer(transfer) => write!(f, "SentTransfer({:?})", transfer),
            Response::SentTransfers(transfers) => write!(f, "SentTransfers({:?})", transfers),
            Response::StrongholdPasswordIsAvailable(is_available) => {
                write!(f, "StrongholdPasswordIsAvailable({:?})", is_available)
            }
            Response::Error(error) => write!(f, "Error({:?})", error),
            Response::Panic(panic_msg) => write!(f, "Panic({:?})", panic_msg),
            Response::GeneratedMnemonic(_) => write!(f, "GeneratedMnemonic(<omitted>)"),
            Response::NodeInfo(info) => write!(f, "NodeInfo({:?})", info),
            Response::Ok(()) => write!(f, "Ok(())"),
        }
    }
}

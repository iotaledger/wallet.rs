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
    /// Account succesfully created or GetAccount response.
    Account(AccountDto),
    /// GetAccounts response.
    Accounts(Vec<AccountDto>),
    /// ListAddresses
    Addresses(Vec<AccountAddress>),
    /// ListAddressesWithUnspentOutputs.
    AddressesWithUnspentOutputs(Vec<AddressWithUnspentOutputsDto>),
    /// GetOutputsWithAdditionalUnlockConditions.
    OutputIds(Vec<OutputId>),
    /// GetOutput.
    Output(Box<Option<OutputData>>),
    /// ListOutputs/ListUnspentOutputs.
    Outputs(Vec<OutputData>),
    /// ListTransactions/ListPendingTransactions.
    Transactions(Vec<Transaction>),
    /// GenerateAddress response.
    GeneratedAddress(Vec<AccountAddress>),
    /// GetBalance/SyncAccount response.
    Balance(AccountBalanceDto),
    /// SendAmount, MintNativeTokens, MintNfts, SendMicroTransaction, SendNativeTokens, SendNft, SendTransfer and
    /// InternalTransfer response.
    SentTransfer(TransferResult),
    /// TryCollectOutputs and CollectOutputs response.
    SentTransfers(Vec<TransferResult>),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// GenerateMnemonic response.
    GeneratedMnemonic(String),
    /// Node info response.
    NodeInfo(NodeInfoWrapper),
    /// All went fine.
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
            Response::Error(error) => write!(f, "Error({:?})", error),
            Response::Panic(panic_msg) => write!(f, "Panic({:?})", panic_msg),
            Response::GeneratedMnemonic(_) => write!(f, "GeneratedMnemonic(<omitted>)"),
            Response::NodeInfo(info) => write!(f, "NodeInfo({:?})", info),
            Response::Ok(()) => write!(f, "Ok(())"),
        }
    }
}

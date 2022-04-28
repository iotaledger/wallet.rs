// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{bee_message::output::OutputId, NodeInfoWrapper};
use serde::Serialize;

use crate::{
    account::{
        operations::transfer::TransferResult,
        types::{
            address::{AccountAddress, AddressWithUnspentOutputs},
            AccountBalance, OutputData, Transaction,
        },
        Account,
    },
    Error,
};

/// The response message.
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseType {
    /// Account succesfully created or GetAccount response.
    Account(Account),
    /// GetAccounts response.
    Accounts(Vec<Account>),
    /// ListAddresses
    Addresses(Vec<AccountAddress>),
    /// ListAddressesWithUnspentOutputs.
    AddressesWithUnspentOutputs(Vec<AddressWithUnspentOutputs>),
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
    Balance(AccountBalance),
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

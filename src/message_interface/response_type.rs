// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        operations::transfer::TransferResult,
        types::{
            address::{AccountAddress, AddressWithBalance},
            AccountBalance, OutputData, Transaction,
        },
        Account,
    },
    Error,
};

use serde::Serialize;

/// The response message.
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ResponseType {
    /// Account succesfully created.
    CreatedAccount(Account),
    /// GetAccount response.
    ReadAccount(Account),
    /// GetAccounts response.
    ReadAccounts(Vec<Account>),
    /// ListAddresses
    Addresses(Vec<AccountAddress>),
    /// ListAddressesWithBalance.
    AddressesWithBalance(Vec<AddressWithBalance>),
    /// ListOutputs/ListUnspentOutputs.
    Outputs(Vec<OutputData>),
    /// ListTransactions/ListPendingTransactions.
    Transactions(Vec<Transaction>),
    /// GenerateAddress response.
    GeneratedAddress(Vec<AccountAddress>),
    /// GetBalance response.
    Balance(AccountBalance),
    /// SyncAccount response.
    SyncedAccount(AccountBalance),
    /// Backup response.
    BackupSuccessful,
    /// ImportAccounts response.
    BackupRestored,
    /// SendTransfer and InternalTransfer response.
    SentTransfer(TransferResult),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// GenerateMnemonic response.
    GeneratedMnemonic(String),
    /// VerifyMnemonic response.
    VerifiedMnemonic,
    // /// SetAlias response.
    // UpdatedAlias,
    /// DeleteStorage response.
    DeletedStorage,
    /// SetClientOptions response.
    UpdatedAllClientOptions,
    /// All went fine.
    Ok(()),
}

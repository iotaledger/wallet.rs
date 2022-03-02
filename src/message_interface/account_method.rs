// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::operations::{
    address_generation::AddressGenerationOptions, syncing::SyncOptions, transfer::TransferOptions,
};
use iota_client::bee_message::output::Output;

use serde::Deserialize;

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// List addresses.
    ListAddresses,
    /// Returns only addresses of the account with balance
    ListAddressesWithBalance,
    /// Returns all outputs of the account
    ListOutputs,
    /// Returns all unspent outputs of the account
    ListUnspentOutputs,
    /// Returns all transaction of the account
    ListTransactions,
    /// Returns all pending transaction of the account
    ListPendingTransactions,
    /// Get account balance information.
    GetBalance,
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send funds.
    SendTransfer {
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
    },
}

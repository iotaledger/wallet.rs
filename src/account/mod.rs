// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The module with the AccountBuilder.
pub(crate) mod builder;
/// Constants used for the account and account operations.
pub(crate) mod constants;
/// A thread guard over an account, all account methods are called from here.
pub(crate) mod handle;
/// The account operations like address generation, syncing and creating transfers.
pub(crate) mod operations;
/// Types used in an account and returned from methods.
pub mod types;

use std::collections::{HashMap, HashSet};

use getset::{Getters, Setters};
use iota_client::bee_message::{output::OutputId, payload::transaction::TransactionId};
use serde::{Deserialize, Serialize};

use self::types::{
    address::{AccountAddress, AddressWithUnspentOutputs},
    AccountBalance, OutputData,
};
pub use self::{
    handle::AccountHandle,
    operations::{
        address_generation::AddressGenerationOptions,
        output_collection::OutputsToCollect,
        syncing::SyncOptions,
        transfer::{RemainderValueStrategy, TransferOptions},
    },
};

/// An Account.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// The account index
    index: u32,
    /// The coin type
    coin_type: u32,
    /// The account alias.
    alias: String,
    /// Public addresses
    pub(crate) public_addresses: Vec<AccountAddress>,
    /// Internal addresses
    pub(crate) internal_addresses: Vec<AccountAddress>,
    /// Addresses with unspent outputs
    // used to improve performance for syncing and getbalance because it's in most cases only a subset of all addresses
    addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    /// Outputs
    // stored separated from the account for performance?
    outputs: HashMap<OutputId, OutputData>,
    /// Unspent outputs that are currently used as input for transactions
    // outputs used in transactions should be locked here so they don't get used again, which would result in a
    // conflicting transaction
    locked_outputs: HashSet<OutputId>,
    /// Unspent outputs
    // have unspent outputs in a separated hashmap so we don't need to iterate over all outputs we have
    unspent_outputs: HashMap<OutputId, OutputData>,
    /// Sent transactions
    // stored separated from the account for performance and only the transaction id here? where to add the network id?
    // transactions: HashSet<TransactionId>,
    transactions: HashMap<TransactionId, types::Transaction>,
    /// Pending transactions
    // Maybe pending transactions even additionally separated?
    pending_transactions: HashSet<TransactionId>,
    /// Account options
    // sync interval, output consolidation
    #[getset(get = "pub(crate)")]
    account_options: AccountOptions,
}

/// Account options
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountOptions {
    /// Threshold for the amount of unspent outputs before they get consolidated (sent in a transaction to the account
    /// itself)
    pub output_consolidation_threshold: usize,
    pub automatic_output_consolidation: bool,
    // #[cfg(feature = "storage")]
    // pub(crate) persist_events: bool,
}

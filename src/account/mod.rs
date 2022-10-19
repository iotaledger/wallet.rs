// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The module with the AccountBuilder.
pub(crate) mod builder;
/// Constants used for the account and account operations.
pub(crate) mod constants;
/// A thread guard over an account, all account methods are called from here.
pub(crate) mod handle;
/// The account operations like address generation, syncing and creating transactions.
pub(crate) mod operations;
/// Types used in an account and returned from methods.
pub mod types;
/// Methods to update the account state.
pub(crate) mod update;

use std::collections::{HashMap, HashSet};

use getset::{Getters, Setters};
use iota_client::{
    api_types::response::OutputResponse,
    block::{
        output::OutputId,
        payload::{transaction::TransactionId, TransactionPayload},
    },
};
use serde::{Deserialize, Serialize};

use self::types::{
    address::{AccountAddress, AddressWithUnspentOutputs},
    AccountBalance, OutputData,
};
pub use self::{
    handle::AccountHandle,
    operations::{
        address_generation::AddressGenerationOptions,
        output_claiming::OutputsToClaim,
        syncing::SyncOptions,
        transaction::{
            prepare_output::{Assets, Features, OutputOptions, StorageDeposit, Unlocks},
            RemainderValueStrategy, TransactionOptions,
        },
    },
    types::OutputDataDto,
};

/// An Account.
#[derive(Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Account {
    /// The account index
    index: u32,
    /// The coin type
    #[serde(rename = "coinType")]
    coin_type: u32,
    /// The account alias.
    alias: String,
    /// Public addresses
    #[serde(rename = "publicAddresses")]
    pub(crate) public_addresses: Vec<AccountAddress>,
    /// Internal addresses
    #[serde(rename = "internalAddresses")]
    pub(crate) internal_addresses: Vec<AccountAddress>,
    /// Addresses with unspent outputs
    // used to improve performance for syncing and getbalance because it's in most cases only a subset of all addresses
    #[serde(rename = "addressesWithUnspentOutputs")]
    addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    /// Outputs
    // stored separated from the account for performance?
    outputs: HashMap<OutputId, OutputData>,
    /// Unspent outputs that are currently used as input for transactions
    // outputs used in transactions should be locked here so they don't get used again, which would result in a
    // conflicting transaction
    #[serde(rename = "lockedOutputs")]
    locked_outputs: HashSet<OutputId>,
    /// Unspent outputs
    // have unspent outputs in a separated hashmap so we don't need to iterate over all outputs we have
    #[serde(rename = "unspentOutputs")]
    unspent_outputs: HashMap<OutputId, OutputData>,
    /// Sent transactions
    // stored separated from the account for performance and only the transaction id here? where to add the network id?
    // transactions: HashSet<TransactionId>,
    transactions: HashMap<TransactionId, types::Transaction>,
    /// Pending transactions
    // Maybe pending transactions even additionally separated?
    #[serde(rename = "pendingTransactions")]
    pending_transactions: HashSet<TransactionId>,
    /// Transaction payloads for received outputs with inputs when not pruned before syncing, can be used to determine
    /// the sender address/es
    #[serde(rename = "incomingTransactions")]
    incoming_transactions: HashMap<TransactionId, (TransactionPayload, Vec<OutputResponse>)>,
}

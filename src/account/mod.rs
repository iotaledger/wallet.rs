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
pub use operations::{
    address_generation::AddressGenerationOptions,
    transfer::{RemainderValueStrategy, TransferOptions, TransferOutput},
};

use crate::{
    account::types::{
        address::{AccountAddress, AddressWithBalance},
        AccountBalance, OutputData,
    },
    signing::SignerType,
};

use getset::{Getters, Setters};
use iota_client::bee_message::{output::OutputId, payload::transaction::TransactionId};
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

/// An Account.
#[derive(Debug, Getters, Setters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Account {
    /// The account identifier.
    #[getset(set = "pub(crate)")]
    id: String,
    /// The account index
    index: usize,
    /// The account alias.
    alias: String,
    /// The account's signer type.
    #[serde(rename = "signerType")]
    signer_type: SignerType,
    pub(crate) public_addresses: Vec<AccountAddress>,
    pub(crate) internal_addresses: Vec<AccountAddress>,
    // used to improve performance for syncing and getbalance because it's in most cases only a subset of all addresses
    addresses_with_balance: Vec<AddressWithBalance>,
    // stored separated from the account for performance?
    outputs: HashMap<OutputId, OutputData>,
    // outputs used in transactions should be locked here so they don't get used again, resulting in conflicting
    // transactions
    locked_outputs: HashSet<OutputId>,
    // have unspent outputs in a separated hashmap so we don't need to iterate over all outputs we have
    unspent_outputs: HashMap<OutputId, OutputData>,
    // stored separated from the account for performance and only the transaction id here? where to add the network id?
    // transactions: HashSet<TransactionId>,
    transactions: HashMap<TransactionId, types::Transaction>,
    // Maybe pending transactions even additionally separated?
    pending_transactions: HashSet<TransactionId>,
    // sync interval, output consolidation
    #[getset(get = "pub(crate)")]
    account_options: AccountOptions,
}

/// Account options
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct AccountOptions {
    pub(crate) output_consolidation_threshold: usize,
    pub(crate) automatic_output_consolidation: bool,
    /* #[cfg(feature = "storage")]
     * pub(crate) persist_events: bool, */
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::account::operations::output_collection::OutputsToCollect;

/// The synchronization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    /// Specific Bech32 encoded addresses of the account to sync, if addresses are provided, then `address_start_index`
    /// will be ignored
    #[serde(rename = "addresses", default)]
    pub addresses: Vec<String>,
    /// Address index from which to start syncing addresses. 0 by default, using a higher index will be faster because
    /// addresses with a lower index will be skipped, but could result in a wrong balance for that reason
    #[serde(rename = "addressStartIndex", default = "default_address_start_index")]
    pub address_start_index: u32,
    #[serde(
        rename = "automaticOutputConsolidation",
        default = "default_automatic_output_consolidation"
    )]
    pub automatic_output_consolidation: bool,
    /// Usually we skip syncing if it's called within a few seconds, because there can only be new changes every 10
    /// seconds. But if we change the client options, we need to resync, because the new node could be from a nother
    /// network and then we need to check all addresses. This will also ignore `address_start_index` and sync all
    /// addresses.
    #[serde(rename = "forceSyncing", default)]
    pub force_syncing: bool,
    /// Checks pending transactions and promotes/reattaches them if necessary.
    #[serde(rename = "syncTransactions", default = "default_sync_pending_transactions")]
    pub sync_pending_transactions: bool,
    /// Specifies if only basic outputs should be synced or also alias and nft outputs
    #[serde(rename = "syncAliasesAndNfts", default = "default_sync_aliases_and_nfts")]
    pub sync_aliases_and_nfts: bool,
    // Automatically try to collect basic outputs that have additional unlock conditions to their
    // [AddressUnlockCondition](iota_client::bee_block::output::unlock_condition::AddressUnlockCondition).
    #[serde(rename = "tryCollectOutputs", default = "default_try_collect_outputs")]
    pub try_collect_outputs: OutputsToCollect,
    /// Amount of unspent outputs, only with a
    /// [`AddressUnlockCondition`](iota_client::bee_block::output::unlock_condition::AddressUnlockCondition),
    /// before they get consolidated (merged with a transaction to a single output).
    #[serde(
        rename = "outputConsolidationThreshold",
        default = "default_output_consolidation_threshold"
    )]
    pub output_consolidation_threshold: usize,
}

fn default_output_consolidation_threshold() -> usize {
    100
}

fn default_automatic_output_consolidation() -> bool {
    false
}

fn default_sync_pending_transactions() -> bool {
    true
}

fn default_sync_aliases_and_nfts() -> bool {
    true
}

fn default_try_collect_outputs() -> OutputsToCollect {
    OutputsToCollect::None
}

fn default_address_start_index() -> u32 {
    0
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            addresses: Vec::new(),
            output_consolidation_threshold: 100,
            automatic_output_consolidation: false,
            address_start_index: 0,
            sync_pending_transactions: true,
            sync_aliases_and_nfts: true,
            try_collect_outputs: OutputsToCollect::None,
            force_syncing: false,
        }
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// The synchronization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    #[serde(
        rename = "outputConsolidationThreshold",
        default = "default_output_consolidation_threshold"
    )]
    pub output_consolidation_threshold: usize,
    #[serde(
        rename = "automaticOutputConsolidation",
        default = "default_automatic_output_consolidation"
    )]
    pub automatic_output_consolidation: bool,
    // 0 by default, using a higher value will be faster, but could result in a wrong balance, since addresses with a
    // lower index aren't synced
    #[serde(rename = "addressStartIndex", default = "default_address_start_index")]
    pub address_start_index: u32,
    // 0 by default, no new address should be generated during syncing
    #[serde(rename = "gapLimit", default = "default_gap_limit")]
    pub gap_limit: usize,
    #[serde(rename = "syncTransactions", default = "default_sync_pending_transactions")]
    pub sync_pending_transactions: bool,
    #[serde(rename = "syncAliasesAndNfts", default = "default_sync_aliases_and_nfts")]
    pub sync_aliases_and_nfts: bool,
    // Syncs all addresses of the account and not only the ones with balance (required when syncing the account in a
    // new network, because addresses that had balance in the old network, but not in the new one, wouldn't get
    // updated)
    #[serde(rename = "syncAllAddresses", default)]
    pub sync_all_addresses: bool,
    // usually we skip syncing if it's called within a few ms, but if we change the client options we need to resync
    // this will also ignore `address_start_index` and sync all addresses
    #[serde(rename = "forceSyncing", default)]
    pub force_syncing: bool,
}

fn default_output_consolidation_threshold() -> usize {
    100
}

fn default_automatic_output_consolidation() -> bool {
    true
}

fn default_sync_pending_transactions() -> bool {
    true
}

fn default_sync_aliases_and_nfts() -> bool {
    true
}

fn default_address_start_index() -> u32 {
    0
}

fn default_gap_limit() -> usize {
    0
}

impl Default for SyncOptions{
    fn default()-> Self{
        Self{
            output_consolidation_threshold: 100,
            automatic_output_consolidation: true,
            address_start_index: 0,
            gap_limit: 0,
            sync_pending_transactions: true,
            sync_aliases_and_nfts: true,
            sync_all_addresses: false,
            force_syncing: false,

        }
    }
}
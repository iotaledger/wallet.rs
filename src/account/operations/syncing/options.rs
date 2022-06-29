// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

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
    /// Usually we skip syncing if it's called within a few seconds, because there can only be new changes every 5
    /// seconds. But if we change the client options, we need to resync, because the new node could be from a nother
    /// network and then we need to check all addresses. This will also ignore `address_start_index` and sync all
    /// addresses.
    #[serde(rename = "forceSyncing", default)]
    pub force_syncing: bool,
    /// Try to sync transactions from incoming outputs with their inputs. Some data may not be obtained if it has been
    /// pruned.
    #[serde(rename = "syncIncomingTransactions", default = "default_sync_incoming_transactions")]
    pub sync_incoming_transactions: bool,
    /// Checks pending transactions and promotes/reattaches them if necessary.
    #[serde(rename = "syncPendingTransactions", default = "default_sync_pending_transactions")]
    pub sync_pending_transactions: bool,
    /// Specifies if only basic outputs should be synced or also alias and nft outputs
    #[serde(rename = "syncAliasesAndNfts", default = "default_sync_aliases_and_nfts")]
    pub sync_aliases_and_nfts: bool,
}

fn default_sync_incoming_transactions() -> bool {
    false
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

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            addresses: Vec::new(),
            address_start_index: default_address_start_index(),
            sync_incoming_transactions: default_sync_incoming_transactions(),
            sync_pending_transactions: default_sync_pending_transactions(),
            sync_aliases_and_nfts: default_sync_aliases_and_nfts(),
            force_syncing: false,
        }
    }
}

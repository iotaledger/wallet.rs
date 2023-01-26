// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

const DEFAULT_ADDRESS_START_INDEX: u32 = 0;
const DEFAULT_FORCE_SYNCING: bool = false;
const DEFAULT_SYNC_INCOMING_TRANSACTIONS: bool = false;
const DEFAULT_SYNC_ONLY_MOST_BASIC_OUTPUTS: bool = false;
const DEFAULT_SYNC_PENDING_TRANSACTIONS: bool = true;
const DEFAULT_SYNC_NATIVE_TOKEN_FOUNDRIES: bool = false;

/// The synchronization options
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SyncOptions {
    /// Specific Bech32 encoded addresses of the account to sync, if addresses are provided, then `address_start_index`
    /// will be ignored
    #[serde(rename = "addresses", default)]
    pub addresses: Vec<String>,
    /// Address index from which to start syncing addresses. 0 by default, using a higher index will be faster because
    /// addresses with a lower index will be skipped, but could result in a wrong balance for that reason
    #[serde(rename = "addressStartIndex", default = "default_address_start_index")]
    pub address_start_index: u32,
    /// Address index from which to start syncing internal addresses. 0 by default, using a higher index will be faster
    /// because addresses with a lower index will be skipped, but could result in a wrong balance for that reason
    #[serde(rename = "addressStartIndexInternal", default = "default_address_start_index")]
    pub address_start_index_internal: u32,
    /// Usually syncing is skipped if it's called in between 200ms, because there can only be new changes every
    /// milestone and calling it twice "at the same time" will not return new data
    /// When this to true, we will sync anyways, even if it's called 0ms after the las sync finished.
    #[serde(rename = "forceSyncing", default)]
    pub force_syncing: bool,
    /// Try to sync transactions from incoming outputs with their inputs. Some data may not be obtained if it has been
    /// pruned.
    #[serde(rename = "syncIncomingTransactions", default = "default_sync_incoming_transactions")]
    pub sync_incoming_transactions: bool,
    /// Checks pending transactions and promotes/reattaches them if necessary.
    #[serde(rename = "syncPendingTransactions", default = "default_sync_pending_transactions")]
    pub sync_pending_transactions: bool,
    /// Specifies what outputs should be synced for the ed25519 addresses from the account.
    #[serde(default)]
    pub account: AccountSyncOptions,
    /// Specifies what outputs should be synced for the address of an alias output.
    #[serde(default)]
    pub alias: AliasSyncOptions,
    /// Specifies what outputs should be synced for the address of an nft output.
    #[serde(default)]
    pub nft: NftSyncOptions,
    /// Specifies if only basic outputs with an AddressUnlockCondition alone should be synced, will overwrite
    /// `account`, `alias` and `nft` options.
    #[serde(
        rename = "syncOnlyMostBasicOutputs",
        default = "default_sync_only_most_basic_outputs"
    )]
    pub sync_only_most_basic_outputs: bool,
    /// Sync native token foundries, so their metadata can be returned in the balance.
    #[serde(rename = "syncNativeTokenFoundries", default = "default_sync_native_token_foundries")]
    pub sync_native_token_foundries: bool,
}

fn default_address_start_index() -> u32 {
    DEFAULT_ADDRESS_START_INDEX
}

fn default_force_syncing() -> bool {
    DEFAULT_FORCE_SYNCING
}

fn default_sync_incoming_transactions() -> bool {
    DEFAULT_SYNC_INCOMING_TRANSACTIONS
}

fn default_sync_only_most_basic_outputs() -> bool {
    DEFAULT_SYNC_ONLY_MOST_BASIC_OUTPUTS
}

fn default_sync_pending_transactions() -> bool {
    DEFAULT_SYNC_PENDING_TRANSACTIONS
}

fn default_sync_native_token_foundries() -> bool {
    DEFAULT_SYNC_NATIVE_TOKEN_FOUNDRIES
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            addresses: Vec::new(),
            address_start_index: default_address_start_index(),
            address_start_index_internal: default_address_start_index(),
            sync_incoming_transactions: default_sync_incoming_transactions(),
            sync_pending_transactions: default_sync_pending_transactions(),
            account: AccountSyncOptions::default(),
            alias: AliasSyncOptions::default(),
            nft: NftSyncOptions::default(),
            sync_only_most_basic_outputs: default_sync_only_most_basic_outputs(),
            sync_native_token_foundries: default_sync_native_token_foundries(),
            force_syncing: default_force_syncing(),
        }
    }
}

/// Sync options for Ed25519 addresses from the account
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AccountSyncOptions {
    pub basic_outputs: bool,
    pub nft_outputs: bool,
    pub alias_outputs: bool,
}

impl Default for AccountSyncOptions {
    fn default() -> Self {
        // Sync only basic outputs
        Self {
            basic_outputs: true,
            nft_outputs: false,
            alias_outputs: false,
        }
    }
}

/// Sync options for addresses from alias outputs
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AliasSyncOptions {
    pub basic_outputs: bool,
    pub nft_outputs: bool,
    pub alias_outputs: bool,
    pub foundry_outputs: bool,
}

impl Default for AliasSyncOptions {
    // Sync only foundries
    fn default() -> Self {
        Self {
            basic_outputs: false,
            nft_outputs: false,
            alias_outputs: false,
            foundry_outputs: true,
        }
    }
}

/// Sync options for addresses from NFT outputs
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NftSyncOptions {
    pub basic_outputs: bool,
    pub nft_outputs: bool,
    pub alias_outputs: bool,
}

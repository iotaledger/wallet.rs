// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Amount at which outputs on a single addresses will get consolidated by default if consolidatioin is enabled
pub(crate) const DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD: usize = 100;
/// Amount at which outputs on a single addresses will get consolidated by default with a ledger signer if
/// consolidatioin is enabled, needs to be smaller because the memory of the ledger nano s is limited
pub(crate) const DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD: usize = 16;

/// Amount of API request that can be sent in parallel during syncing
pub(crate) const PARALLEL_REQUESTS_AMOUNT: usize = 500;

/// ms before an account actually syncs with the network, before it just returns the previous syncing result
/// this is done to prevent unnecessary simultaneous synchronizations
pub(crate) const MIN_SYNC_INTERVAL: u128 = 5000;

/// The minimum amount of IOTA tokens an address with outputs needs to have
pub(crate) const MIN_DUST_ALLOWANCE_VALUE: u64 = 1_000_000;

/// The account prefix (do we really need this?)
pub(crate) const ACCOUNT_ID_PREFIX: &str = "wallet-account://";

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The module for the address generation
pub(crate) mod address_generation;
/// The module to get the accounts balance
pub(crate) mod balance;
/// Helper functions
pub(crate) mod helpers;
/// The module for claiming of outputs with
/// [`UnlockCondition`](iota_client::block::output::UnlockCondition)s that aren't only
/// [`AddressUnlockCondition`](iota_client::block::output::unlock_condition::AddressUnlockCondition)
pub(crate) mod output_claiming;
/// The module for the output consolidation
pub(crate) mod output_consolidation;
/// The module to find additional addresses with unspent outputs
pub(crate) mod output_finder;
/// The module for participation
#[cfg(all(feature = "participation", feature = "storage"))]
pub(crate) mod participation;
/// The module for synchronization of an account
pub(crate) mod syncing;
/// The module for transactions
pub(crate) mod transaction;

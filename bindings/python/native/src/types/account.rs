// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota::bee_rest_api::types::responses::InfoResponse as RustInfoResponse;
use iota_wallet::{
    account::{
        AccountBalance as RustAccountBalance, AccountHandle as RustAccountHandle,
        AccountInitialiser as RustAccountInitialiser, AccountSynchronizer as RustAccountSynchronizer,
        SyncedAccount as RustSyncedAccount,
    },
    account_manager::{
        AccountManager as RustAccountManager, AccountStore, AccountsSynchronizer as RustAccountsSynchronizer,
    },
    address::Address as RustWalletAddress,
    message::Transfer as RustTransfer,
};
use pyo3::prelude::*;
use std::convert::{From, Into};

#[pyclass]
pub struct AccountManager {
    pub account_manager: RustAccountManager,
}

#[pyclass]
pub struct AccountInitialiser {
    pub account_initialiser: Option<RustAccountInitialiser>,
    pub addresses: Vec<RustWalletAddress>,
    pub accounts: AccountStore,
}

#[pyclass]
pub struct AccountHandle {
    pub account_handle: RustAccountHandle,
}

#[pyclass]
pub struct SyncedAccount {
    pub synced_account: RustSyncedAccount,
}

#[pyclass]
pub struct AccountSynchronizer {
    pub account_synchronizer: Option<RustAccountSynchronizer>,
}

#[pyclass]
pub struct AccountsSynchronizer {
    pub accounts_synchronizer: Option<RustAccountsSynchronizer>,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Transfer {
    pub transfer: RustTransfer,
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct AccountBalance {
    /// Account's total balance.
    pub total: u64,
    // The available balance is the balance users are allowed to spend.
    /// For example, if a user with 50i total account balance has made a message spending 20i,
    /// the available balance should be (50i-30i) = 20i.
    pub available: u64,
    /// Balances from message with `incoming: true`.
    /// Note that this may not be accurate since the node prunes the messags.
    pub incoming: u64,
    /// Balances from message with `incoming: false`.
    /// Note that this may not be accurate since the node prunes the messags.
    pub outgoing: u64,
}

impl From<RustAccountBalance> for AccountBalance {
    fn from(acount_balance: RustAccountBalance) -> Self {
        Self {
            total: acount_balance.total,
            available: acount_balance.available,
            incoming: acount_balance.incoming,
            outgoing: acount_balance.outgoing,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub is_healthy: bool,
    pub network_id: String,
    pub bech32_hrp: String,
    pub latest_milestone_index: u32,
    pub confirmed_milestone_index: u32,
    pub pruning_index: u32,
    pub features: Vec<String>,
    pub min_pow_score: f64,
}

impl From<RustInfoResponse> for InfoResponse {
    fn from(info: RustInfoResponse) -> Self {
        InfoResponse {
            name: info.name,
            version: info.version,
            is_healthy: info.is_healthy,
            network_id: info.network_id,
            bech32_hrp: info.bech32_hrp,
            latest_milestone_index: info.latest_milestone_index,
            confirmed_milestone_index: info.confirmed_milestone_index,
            pruning_index: info.pruning_index,
            features: info.features,
            min_pow_score: info.min_pow_score,
        }
    }
}

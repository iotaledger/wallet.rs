// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
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

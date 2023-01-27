// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::{Error, Result};

use core::convert::TryFrom;
use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_client::NodeInfoWrapper as RustNodeInfoWrapper;
use iota_wallet::{
    account::{
        AccountBalance as RustAccountBalance, AccountHandle as RustAccountHandle,
        AccountInitialiser as RustAccountInitialiser, AccountSynchronizer as RustAccountSynchronizer,
        SyncedAccount as RustSyncedAccount,
    },
    account_manager::{
        AccountManager as RustAccountManager, AccountStore, AccountsSynchronizer as RustAccountsSynchronizer,
    },
    address::{parse as parse_address, Address as RustWalletAddress, OutputKind as RustOutputKind},
    message::{Transfer as RustTransfer, TransferOutput as RustTransferOutput},
};
use pyo3::prelude::*;
use std::{
    convert::{From, Into},
    num::NonZeroU64,
};

#[pyclass]
pub struct AccountManager {
    pub account_manager: Option<RustAccountManager>,
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

#[pyclass]
#[derive(Debug, Clone)]
pub struct TransferWithOutputs {
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
pub struct TransferOutput {
    pub address: String,
    pub amount: u64,
    pub output_kind: Option<String>,
}

impl TryFrom<TransferOutput> for RustTransferOutput {
    type Error = Error;
    fn try_from(info: TransferOutput) -> Result<Self> {
        let output_kind = match info.output_kind.as_deref() {
            Some("SignatureLockedSingle") => RustOutputKind::SignatureLockedSingle,
            Some("SignatureLockedDustAllowance") => RustOutputKind::SignatureLockedDustAllowance,
            _ => RustOutputKind::SignatureLockedSingle,
        };
        Ok(RustTransferOutput {
            address: parse_address(info.address)?,
            amount: NonZeroU64::new(info.amount).unwrap(),
            output_kind,
        })
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub is_healthy: bool,
    pub network_id: String,
    pub bech32_hrp: String,
    pub min_pow_score: f64,
    pub messages_per_second: f64,
    pub referenced_messages_per_second: f64,
    pub referenced_rate: f64,
    pub latest_milestone_timestamp: u64,
    pub latest_milestone_index: u32,
    pub confirmed_milestone_index: u32,
    pub pruning_index: u32,
    pub features: Vec<String>,
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct NodeInfoWrapper {
    pub nodeinfo: InfoResponse,
    pub url: String,
}

impl From<RustNodeInfoWrapper> for NodeInfoWrapper {
    fn from(info: RustNodeInfoWrapper) -> Self {
        NodeInfoWrapper {
            url: info.url,
            nodeinfo: InfoResponse {
                name: info.nodeinfo.name,
                version: info.nodeinfo.version,
                is_healthy: info.nodeinfo.is_healthy,
                network_id: info.nodeinfo.network_id,
                bech32_hrp: info.nodeinfo.bech32_hrp,
                min_pow_score: info.nodeinfo.min_pow_score,
                messages_per_second: info.nodeinfo.messages_per_second,
                referenced_messages_per_second: info.nodeinfo.referenced_messages_per_second,
                referenced_rate: info.nodeinfo.referenced_rate,
                latest_milestone_timestamp: info.nodeinfo.latest_milestone_timestamp,
                latest_milestone_index: info.nodeinfo.latest_milestone_index,
                confirmed_milestone_index: info.nodeinfo.confirmed_milestone_index,
                pruning_index: info.nodeinfo.pruning_index,
                features: info.nodeinfo.features,
            },
        }
    }
}

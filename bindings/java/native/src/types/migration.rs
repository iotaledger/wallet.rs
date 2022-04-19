// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::{CopyGetters, Getters, Setters};
use iota_wallet::{
    account_manager::{
        MigrationAddress as RustMigrationAddress, MigrationBundle as RustMigrationBundle,
        MigrationData as RustMigrationData,
    },
    iota_migration::{
        client::response::InputData as RustInputData, ternary::T3B1Buf, transaction::bundled::BundledTransactionField,
    },
};

use std::convert::{From, Into};

#[derive(Debug, Getters, CopyGetters, Clone)]
pub struct InputData {
    #[getset(get = "pub")]
    address: String,
    #[getset(get_copy = "pub")]
    security_lvl: u8,
    #[getset(get_copy = "pub")]
    balance: u64,
    #[getset(get_copy = "pub")]
    index: u64,
    #[getset(get_copy = "pub")]
    spent: bool,
    spent_bundlehashes: Vec<String>,
}

impl InputData {
    pub fn spent_bundlehashes(&self) -> Vec<String> {
        self.spent_bundlehashes.clone()
    }
}

impl From<RustInputData> for InputData {
    fn from(input: RustInputData) -> Self {
        Self {
            address: input
                .address
                .to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>(),
            security_lvl: input.security_lvl,
            balance: input.balance,
            index: input.index,
            spent: input.spent,
            spent_bundlehashes: match input.spent_bundlehashes {
                Some(v) => v.clone(),
                None => vec![],
            },
        }
    }
}

impl core::fmt::Display for InputData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "address={}, security_lvl={}, balance={}, index={}, spent={}, spent_bundlehashes=({:?})",
            self.address, self.security_lvl, self.balance, self.index, self.spent, self.spent_bundlehashes
        )
    }
}

#[derive(Debug, Getters, CopyGetters)]
pub struct MigrationData {
    #[getset(get_copy = "pub")]
    balance: u64,
    #[getset(get_copy = "pub")]
    last_checked_address_index: u64,
    #[getset(get_copy = "pub")]
    spent_addresses: bool,
    inputs: Vec<InputData>,
}

impl MigrationData {
    pub fn inputs(&self) -> Vec<InputData> {
        self.inputs.clone()
    }
}

impl From<RustMigrationData> for MigrationData {
    fn from(migration_data: RustMigrationData) -> Self {
        Self {
            balance: migration_data.balance,
            last_checked_address_index: migration_data.last_checked_address_index,
            spent_addresses: migration_data.spent_addresses,
            inputs: migration_data.inputs.into_iter().map(Into::into).collect(),
        }
    }
}

impl core::fmt::Display for MigrationData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "balance={}, last_checked_address_index={}, spent_addresses={}, inputs=({:?})",
            self.balance, self.last_checked_address_index, self.spent_addresses, self.inputs
        )
    }
}

#[derive(Debug, Getters, CopyGetters)]
pub struct MigrationBundle {
    #[getset(get_copy = "pub")]
    crackability: f64,
    #[getset(get = "pub")]
    bundle_hash: String,
}

impl From<RustMigrationBundle> for MigrationBundle {
    fn from(migration_bundle: RustMigrationBundle) -> Self {
        Self {
            crackability: *migration_bundle.crackability(),
            bundle_hash: migration_bundle.bundle_hash().clone(),
        }
    }
}

impl core::fmt::Display for MigrationBundle {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "crackability={}, bundle_hash={}",
            self.crackability, self.bundle_hash
        )
    }
}

#[derive(Debug, Getters)]
pub struct MigrationAddress {
    #[getset(get = "pub")]
    pub trytes: String,
    #[getset(get = "pub")]
    pub bech32: String,
}

impl From<RustMigrationAddress> for MigrationAddress {
    fn from(migration_address: RustMigrationAddress) -> Self {
        Self {
            trytes: migration_address.trytes.to_string(),
            bech32: migration_address.bech32.to_string(),
        }
    }
}

impl core::fmt::Display for MigrationAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "trytes={}, bech32={}", self.trytes, self.bech32)
    }
}

#[derive(Default, Debug, Getters, Setters, CopyGetters)]
pub struct MigrationBundleOptions {
    #[getset(get_copy = "pub")]
    mine: bool,
    timeout_secs: Option<u64>,
    #[getset(get_copy = "pub")]
    offset: Option<i64>,
    log_file_name: Option<String>,
}

// Cant use the setters since they return &mut, which is not supported
impl MigrationBundleOptions {
    pub fn set_timeouts(&mut self, secs: i64) {
        self.timeout_secs = if secs >= 0 { Some(secs as u64) } else { None }
    }
    pub fn timeouts(&self) -> u64 {
        match self.timeout_secs {
            Some(t) => t,
            None => 10 * 60,
        }
    }
    pub fn set_mine(&mut self, mine: bool) {
        self.mine = mine
    }
    pub fn set_offset(&mut self, offset: Option<i64>) {
        self.offset = offset
    }
    pub fn set_log_file_name(&mut self, log_file_name: &str) {
        self.log_file_name = Some(log_file_name.to_string())
    }
    pub fn log_file_name(&self) -> Option<String> {
        match &self.log_file_name {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }
}

impl core::fmt::Display for MigrationBundleOptions {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "mine={}, timeout_secs={:?}, offset={:?}, log_file_name={:?}",
            self.mine, self.timeout_secs, self.offset, self.log_file_name
        )
    }
}

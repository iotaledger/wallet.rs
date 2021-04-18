// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_wallet::{
    account_manager::{MigrationBundle as RustMigrationBundle, MigrationData as RustMigrationData},
    iota_migration::{
        client::response::InputData as RustInputData, ternary::T3B1Buf, transaction::bundled::BundledTransactionField,
    },
};
use std::convert::{From, Into};

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct InputData {
    address: String,
    security_lvl: u8,
    balance: u64,
    index: u64,
    spent: bool,
    spent_bundlehashes: Option<Vec<String>>,
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
            spent_bundlehashes: input.spent_bundlehashes,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct MigrationData {
    balance: u64,
    last_checked_address_index: u64,
    inputs: Vec<InputData>,
}

impl From<RustMigrationData> for MigrationData {
    fn from(migration_data: RustMigrationData) -> Self {
        Self {
            balance: migration_data.balance,
            last_checked_address_index: migration_data.last_checked_address_index,
            inputs: migration_data.inputs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct MigrationBundle {
    crackability: f64,
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

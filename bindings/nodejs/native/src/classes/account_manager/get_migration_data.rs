// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use iota_wallet::{
    account_manager::{AccountManager, MigrationData, MigrationDataFinder},
    iota_migration::{ternary::T3B1Buf, transaction::bundled::BundledTransactionField},
    Error,
};
use neon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Default, Deserialize)]
pub struct GetMigrationDataOptions {
    permanode: Option<String>,
    #[serde(rename = "securityLevel")]
    security_level: Option<u8>,
    #[serde(rename = "initialAddressIndex")]
    initial_address_index: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct MigrationInputDto {
    address: String,
    #[serde(rename = "securityLevel")]
    security_level: u8,
    balance: u64,
    index: u64,
    spent: bool,
    #[serde(rename = "spentBundleHashes")]
    spent_bundle_hashes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct MigrationDataDto {
    balance: u64,
    #[serde(rename = "lastCheckedAddressIndex")]
    last_checked_address_index: u64,
    inputs: Vec<MigrationInputDto>,
}

impl From<MigrationData> for MigrationDataDto {
    fn from(data: MigrationData) -> Self {
        let mut inputs: Vec<MigrationInputDto> = Vec::new();
        for input in data.inputs {
            let address = input
                .address
                .to_inner()
                .encode::<T3B1Buf>()
                .iter_trytes()
                .map(char::from)
                .collect::<String>();
            inputs.push(MigrationInputDto {
                address,
                security_level: input.security_lvl,
                balance: input.balance,
                index: input.index,
                spent: input.spent,
                spent_bundle_hashes: input.spent_bundlehashes,
            });
        }
        Self {
            balance: data.balance,
            last_checked_address_index: data.last_checked_address_index,
            inputs,
        }
    }
}

pub struct GetMigrationDataTask {
    pub manager: Arc<RwLock<AccountManager>>,
    pub nodes: Vec<String>,
    pub seed: String,
    pub options: GetMigrationDataOptions,
}

impl Task for GetMigrationDataTask {
    type Output = MigrationData;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            let nodes: Vec<&str> = self.nodes.iter().map(AsRef::as_ref).collect();
            let mut finder = MigrationDataFinder::new(&nodes, &self.seed)?;
            if let Some(permanode) = &self.options.permanode {
                finder = finder.with_permanode(permanode);
            }
            if let Some(initial_address_index) = self.options.initial_address_index {
                finder = finder.with_initial_address_index(initial_address_index);
            }
            if let Some(security_level) = self.options.security_level {
                finder = finder.with_security_level(security_level);
            }
            manager.get_migration_data(finder).await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &MigrationDataDto::from(val))?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

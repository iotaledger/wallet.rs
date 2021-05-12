// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{sync::Arc, time::Duration};

use iota_wallet::{
    account_manager::{AccountManager, MigrationBundle},
    Error,
};
use neon::prelude::*;
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Default, Deserialize)]
pub struct CreateMigrationBundleOptions {
    #[serde(default)]
    mine: bool,
    #[serde(rename = "timeoutSeconds")]
    timeout_secs: Option<u64>,
    offset: Option<i64>,
    #[serde(rename = "logFileName")]
    log_file_name: Option<String>,
}

pub struct CreateMigrationBundleTask {
    pub manager: Arc<RwLock<AccountManager>>,
    pub seed: String,
    pub input_address_indexes: Vec<u64>,
    pub options: CreateMigrationBundleOptions,
}

impl Task for CreateMigrationBundleTask {
    type Output = MigrationBundle;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            manager
                .create_migration_bundle(
                    &self.seed,
                    &self.input_address_indexes,
                    self.options.mine,
                    Duration::from_secs(self.options.timeout_secs.unwrap_or(10 * 60)),
                    self.options.offset.unwrap_or(0),
                    self.options.log_file_name.as_deref().unwrap_or("migration.log"),
                )
                .await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

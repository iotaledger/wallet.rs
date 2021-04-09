// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use iota_wallet::{account_manager::AccountManager, Error};
use neon::prelude::*;
use serde::Deserialize;
use tokio::sync::RwLock;

const DEFAULT_MWM: u8 = 14;

#[derive(Default, Deserialize)]
pub struct SendMigrationBundleOptions {
    mwm: Option<u8>,
}

pub struct SendMigrationBundleTask {
    pub manager: Arc<RwLock<AccountManager>>,
    pub nodes: Vec<String>,
    pub bundle_hash: String,
    pub options: SendMigrationBundleOptions,
}

impl Task for SendMigrationBundleTask {
    type Output = ();
    type Error = Error;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            let nodes: Vec<&str> = self.nodes.iter().map(AsRef::as_ref).collect();
            manager
                .send_migration_bundle(&nodes, &self.bundle_hash, self.options.mwm.unwrap_or(DEFAULT_MWM))
                .await?;
            Ok(())
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(_) => Ok(cx.undefined()),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

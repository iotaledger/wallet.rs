// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account::SyncedAccount, Error};
use neon::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct SyncOptions {
    #[serde(rename = "addressIndex")]
    address_index: Option<usize>,
    #[serde(rename = "gapLimit")]
    gap_limit: Option<usize>,
}

pub struct SyncTask {
    pub account_id: String,
    pub options: SyncOptions,
}

impl Task for SyncTask {
    type Output = SyncedAccount;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(async move {
            let account = crate::get_account(&self.account_id).await;
            let mut synchronizer = account.sync().await;
            if let Some(address_index) = self.options.address_index {
                synchronizer = synchronizer.address_index(address_index);
            }
            if let Some(gap_limit) = self.options.gap_limit {
                synchronizer = synchronizer.gap_limit(gap_limit);
            }
            synchronizer.execute().await
        })
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => {
                let id = crate::block_on(crate::store_synced_account(val));
                let id = cx.string(id);
                Ok(crate::JsSyncedAccount::new(&mut cx, vec![id])?.upcast())
            }
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

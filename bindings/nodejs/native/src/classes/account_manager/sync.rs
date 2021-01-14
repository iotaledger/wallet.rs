// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use iota_wallet::{account::SyncedAccount, account_manager::AccountManager, Error};
use neon::prelude::*;
use tokio::sync::RwLock;

pub struct SyncTask {
    pub manager: Arc<RwLock<AccountManager>>,
}

impl Task for SyncTask {
    type Output = Vec<SyncedAccount>;
    type Error = Error;
    type JsEvent = JsArray;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            manager.sync_accounts().await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(synced_accounts) => {
                let js_array = JsArray::new(&mut cx, synced_accounts.len() as u32);
                for (index, synced_account) in synced_accounts.into_iter().enumerate() {
                    let id = crate::block_on(crate::store_synced_account(synced_account));
                    let id = cx.string(id);
                    let synced_instance = crate::JsSyncedAccount::new(&mut cx, vec![id])?;
                    js_array.set(&mut cx, index as u32, synced_instance)?;
                }

                Ok(js_array)
            }
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

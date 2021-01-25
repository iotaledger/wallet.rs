// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use iota_wallet::{account_manager::AccountManager, Error};
use neon::prelude::*;
use tokio::sync::RwLock;

pub struct IsLatestAddressUnusedTask {
    pub manager: Arc<RwLock<AccountManager>>,
}

impl Task for IsLatestAddressUnusedTask {
    type Output = bool;
    type Error = Error;
    type JsEvent = JsBoolean;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            manager.is_latest_address_unused().await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(flag) => Ok(cx.boolean(flag)),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

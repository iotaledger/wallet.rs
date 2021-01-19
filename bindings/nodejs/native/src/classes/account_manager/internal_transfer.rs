// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{num::NonZeroU64, sync::Arc};

use iota_wallet::{account_manager::AccountManager, message::Message, Error};
use neon::prelude::*;
use tokio::sync::RwLock;

pub struct InternalTransferTask {
    pub manager: Arc<RwLock<AccountManager>>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: NonZeroU64,
}

impl Task for InternalTransferTask {
    type Output = Message;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let manager = self.manager.read().await;
            manager
                .internal_transfer(&self.from_account_id, &self.to_account_id, self.amount)
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

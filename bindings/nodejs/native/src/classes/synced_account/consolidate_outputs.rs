// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{message::Message, Error};
use neon::prelude::*;

pub struct ConsolidateOutputsTask {
    pub synced_account_id: String,
}

impl Task for ConsolidateOutputsTask {
    type Output = Vec<Message>;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let synced = crate::get_synced_account(&self.synced_account_id).await;
            let synced = synced.read().await;
            synced.consolidate_outputs().await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

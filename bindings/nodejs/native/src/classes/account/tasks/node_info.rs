// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota::client::NodeInfoWrapper;
use iota_wallet::Error;
use neon::prelude::*;

pub struct NodeInfoTask {
    pub account_id: String,
    pub url: Option<String>,
}

impl Task for NodeInfoTask {
    type Output = NodeInfoWrapper;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            crate::get_account(&self.account_id)
                .await
                .get_node_info(self.url.as_deref())
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

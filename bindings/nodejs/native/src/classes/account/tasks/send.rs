// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    message::{Message, Transfer},
    Error,
};
use neon::prelude::*;

pub struct SendTask {
    pub account_id: String,
    pub transfer: Transfer,
}

impl Task for SendTask {
    type Output = Message;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            crate::get_account(&self.account_id)
                .await
                .transfer(self.transfer.clone())
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

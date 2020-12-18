// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    message::{Message, Transfer},
    WalletError,
};
use neon::prelude::*;

pub struct SendTask {
    pub synced_account_id: String,
    pub transfer: Transfer,
}

impl Task for SendTask {
    type Output = Message;
    type Error = WalletError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let synced = crate::get_synced_account(&self.synced_account_id);
        let synced = synced.read().unwrap();

        crate::block_on(crate::convert_async_panics(|| async {
            synced.transfer(self.transfer.clone()).await
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

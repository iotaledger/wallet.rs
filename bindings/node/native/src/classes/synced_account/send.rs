// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, RwLock};

use iota_wallet::{
    account::SyncedAccount,
    message::{Message, Transfer},
    WalletError,
};
use neon::prelude::*;

pub struct SendTask {
    pub synced: Arc<RwLock<SyncedAccount>>,
    pub account_id: String,
    pub transfer: Transfer,
}

impl Task for SendTask {
    type Output = Message;
    type Error = WalletError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let synced = self.synced.read().unwrap();
        crate::block_on(crate::convert_async_panics(|| async {
            let res = synced.transfer(self.transfer.clone()).await?;
            crate::update_account(&self.account_id, res.account);
            Ok(res.message)
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

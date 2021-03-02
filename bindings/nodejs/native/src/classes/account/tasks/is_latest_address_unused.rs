// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::Error;
use neon::prelude::*;

pub struct IsLatestAddressUnusedTask {
    pub account_id: String,
}

impl Task for IsLatestAddressUnusedTask {
    type Output = bool;
    type Error = Error;
    type JsEvent = JsBoolean;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(async move {
            crate::get_account(&self.account_id)
                .await
                .is_latest_address_unused()
                .await
        })
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(cx.boolean(val)),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

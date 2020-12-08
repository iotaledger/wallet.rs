// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, RwLock};

use iota_wallet::{account_manager::AccountManager, message::Message, WalletError};
use neon::prelude::*;

pub struct InternalTransferTask {
    pub manager: Arc<RwLock<AccountManager>>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: u64,
}

impl Task for InternalTransferTask {
    type Output = Message;
    type Error = WalletError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let manager = self.manager.read().unwrap();
        crate::block_on(crate::convert_async_panics(|| async {
            let from_account = crate::get_account(&self.from_account_id);
            let from_account = from_account.read().unwrap();
            let to_account = crate::get_account(&self.to_account_id);
            let to_account = to_account.read().unwrap();
            let res = manager
                .internal_transfer(
                    from_account.id().into(),
                    to_account.id().into(),
                    self.amount,
                )
                .await?;

            crate::update_account(&self.from_account_id, res.from_account);
            crate::update_account(&self.to_account_id, res.to_account);

            Ok(res.message)
        }))
    }

    fn complete(
        self,
        mut cx: TaskContext,
        value: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

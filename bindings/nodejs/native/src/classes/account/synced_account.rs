// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU64;

use iota_wallet::{
    address::parse as parse_address,
    message::{IndexationPayload, Message, Transfer},
    Error,
};
use neon::prelude::*;

use super::TransferOptions;

#[derive(Clone)]
pub struct SyncedAccountWrapper(pub String);

impl Drop for SyncedAccountWrapper {
    fn drop(&mut self) {
        crate::block_on(crate::remove_synced_account(&self.0));
    }
}

pub struct SendTask {
    pub synced_account_id: String,
    pub transfer: Transfer,
}

impl Task for SendTask {
    type Output = Message;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let synced = crate::get_synced_account(&self.synced_account_id).await;
            let synced = synced.read().await;
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

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced_account_id = cx.argument::<JsString>(0)?.value();
            Ok(SyncedAccountWrapper(synced_account_id))
        }

        method send(mut cx) {
            let address = cx.argument::<JsString>(0)?.value();
            let amount = cx.argument::<JsNumber>(1)?.value() as u64;
            let (options, cb) = match cx.argument_opt(3) {
                Some(arg) => {
                    let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
                    let options = cx.argument::<JsValue>(2)?;
                    let options = neon_serde::from_value(&mut cx, options)?;
                    (options, cb)
                }
                None => (TransferOptions::default(), cx.argument::<JsFunction>(2)?),
            };

            let mut transfer_builder = Transfer::builder(
                parse_address(address).expect("invalid address format"),
                NonZeroU64::new(amount).expect("amount can't be zero")
            ).with_remainder_value_strategy(options.remainder_value_strategy);
            if let Some(indexation) = options.indexation {
                transfer_builder = transfer_builder.with_indexation(
                    IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be empty")
                );
            }

            let this = cx.this();
            let synced_account_id = cx.borrow(&this, |r| r.0.clone());
            let task = SendTask {
                synced_account_id,
                transfer: transfer_builder.finish(),
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

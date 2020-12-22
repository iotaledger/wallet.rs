// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{num::NonZeroU64, str::FromStr};

use iota_wallet::{
    address::parse as parse_address,
    message::{Indexation, MessageId, RemainderValueStrategy, Transfer},
};
use neon::prelude::*;
use serde::Deserialize;

mod repost;
mod send;

#[derive(Clone)]
pub struct SyncedAccountWrapper(pub String);

impl Drop for SyncedAccountWrapper {
    fn drop(&mut self) {
        crate::remove_synced_account(&self.0);
    }
}

#[derive(Deserialize)]
struct IndexationDto {
    index: String,
    data: Option<Vec<u8>>,
}

#[derive(Default, Deserialize)]
struct TransferOptions {
    #[serde(rename = "remainderValueStrategy", default)]
    remainder_value_strategy: RemainderValueStrategy,
    indexation: Option<IndexationDto>,
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
                    Indexation::new(indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be empty")
                );
            }

            let this = cx.this();
            let synced_account_id = cx.borrow(&this, |r| r.0.clone());
            let task = send::SendTask {
                synced_account_id,
                transfer: transfer_builder.finish(),
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method retry(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let synced_account_id = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced_account_id,
                message_id,
                action: repost::RepostAction::Retry,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method reattach(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let synced_account_id = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced_account_id,
                message_id,
                action: repost::RepostAction::Reattach,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method promote(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let synced_account_id = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced_account_id,
                message_id,
                action: repost::RepostAction::Promote,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use iota_wallet::{
    account::SyncedAccount,
    address::parse as parse_address,
    message::{MessageId, RemainderValueStrategy, Transfer},
};
use neon::prelude::*;

mod repost;
mod send;

#[derive(Clone)]
pub struct SyncedAccountWrapper(Arc<RwLock<SyncedAccount>>, String);

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced = cx.argument::<JsString>(0)?.value();
            let account_id = cx.argument::<JsString>(1)?.value();
            let synced: SyncedAccount = serde_json::from_str(&synced).expect("invalid synced account JSON");
            Ok(SyncedAccountWrapper(Arc::new(RwLock::new(synced)), account_id))
        }

        method send(mut cx) {
            let address = cx.argument::<JsString>(0)?.value();
            let amount = cx.argument::<JsNumber>(1)?.value() as u64;
            let (remainder_value_strategy, cb) = match cx.argument_opt(3) {
                Some(arg) => {
                    let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
                    let remainder_value_strategy = cx.argument::<JsValue>(2)?;
                    let remainder_value_strategy = neon_serde::from_value(&mut cx, remainder_value_strategy)?;
                    (remainder_value_strategy, cb)
                }
                None => (RemainderValueStrategy::ChangeAddress, cx.argument::<JsFunction>(2)?),
            };

            let transfer = Transfer::new(parse_address(address).expect("invalid address format"), amount)
                .remainder_value_strategy(remainder_value_strategy);

            let this = cx.this();
            let instance = cx.borrow(&this, |r| r.clone());
            let task = send::SendTask {
                synced: instance.0,
                account_id: instance.1,
                transfer,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method retry(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let instance = cx.borrow(&this, |r| r.clone());
            let task = repost::RepostTask {
                synced: instance.0,
                account_id: instance.1,
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
            let instance = cx.borrow(&this, |r| r.clone());
            let task = repost::RepostTask {
                synced: instance.0,
                account_id: instance.1,
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
            let instance = cx.borrow(&this, |r| r.clone());
            let task = repost::RepostTask {
                synced: instance.0,
                account_id: instance.1,
                message_id,
                action: repost::RepostAction::Promote,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use std::str::FromStr;
use std::sync::{Arc, RwLock};

use iota_wallet::{
    account::SyncedAccount,
    address::parse as parse_address,
    message::{MessageId, RemainderValueStrategy, Transfer},
};
use neon::prelude::*;

mod repost;
mod send;

pub struct SyncedAccountWrapper(Arc<RwLock<SyncedAccount>>);

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced = cx.argument::<JsString>(0)?.value();
            let synced: SyncedAccount = serde_json::from_str(&synced).expect("invalid synced account JSON");
            Ok(SyncedAccountWrapper(Arc::new(RwLock::new(synced))))
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
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = send::SendTask {
                synced,
                transfer,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method retry(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced,
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
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced,
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
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = repost::RepostTask {
                synced,
                message_id,
                action: repost::RepostAction::Promote,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

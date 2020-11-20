use std::sync::{Arc, Mutex};

use iota_wallet::{account::SyncedAccount, address::parse as parse_address, message::Transfer};
use neon::prelude::*;

mod send;

pub struct SyncedAccountWrapper(Arc<Mutex<SyncedAccount>>);

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced = cx.argument::<JsValue>(0)?;
            let synced: SyncedAccount = neon_serde::from_value(&mut cx, synced)?;
            Ok(SyncedAccountWrapper(Arc::new(Mutex::new(synced))))
        }

        method send(mut cx) {
            let address = cx.argument::<JsString>(0)?.value();
            let amount = cx.argument::<JsNumber>(1)?.value() as u64;
            let cb = cx.argument::<JsFunction>(2)?;

            let transfer = Transfer::new(parse_address(address).expect("invalid address format"), amount);

            let this = cx.this();
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = send::SendTask {
                synced,
                transfer,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use neon::prelude::*;

#[derive(Clone)]
pub struct SyncedAccountWrapper(pub String);

impl Drop for SyncedAccountWrapper {
    fn drop(&mut self) {
        crate::block_on(crate::remove_synced_account(&self.0));
    }
}

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced_account_id = cx.argument::<JsString>(0)?.value();
            Ok(SyncedAccountWrapper(synced_account_id))
        }
    }
}

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

use std::sync::{Arc, RwLock};

use iota_wallet::{account::SyncedAccount, account_manager::AccountManager, WalletError};
use neon::prelude::*;

pub struct SyncTask {
  pub manager: Arc<RwLock<AccountManager>>,
}

impl Task for SyncTask {
  type Output = Vec<SyncedAccount>;
  type Error = WalletError;
  type JsEvent = JsArray;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    let manager = self.manager.read().unwrap();
    crate::block_on(crate::convert_async_panics(|| async {
      manager.sync_accounts().await
    }))
  }

  fn complete(
    self,
    mut cx: TaskContext,
    value: Result<Self::Output, Self::Error>,
  ) -> JsResult<Self::JsEvent> {
    match value {
      Ok(synced_accounts) => {
        let js_array = JsArray::new(&mut cx, synced_accounts.len() as u32);
        for (index, synced_account) in synced_accounts.iter().enumerate() {
          let synced = serde_json::to_string(&synced_account).unwrap();
          let synced = cx.string(synced);
          let synced_instance = crate::JsSyncedAccount::new(&mut cx, vec![synced])?;
          js_array.set(&mut cx, index as u32, synced_instance)?;
        }

        Ok(js_array)
      }
      Err(e) => cx.throw_error(e.to_string()),
    }
  }
}

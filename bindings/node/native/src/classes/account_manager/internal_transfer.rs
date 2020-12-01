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

use iota_wallet::{
  account::AccountIdentifier, account_manager::AccountManager, message::Message, WalletError,
};
use neon::prelude::*;

pub struct InternalTransferTask {
  pub manager: Arc<RwLock<AccountManager>>,
  pub from_account_id: AccountIdentifier,
  pub to_account_id: AccountIdentifier,
  pub amount: u64,
}

impl Task for InternalTransferTask {
  type Output = Message;
  type Error = WalletError;
  type JsEvent = JsValue;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    let manager = self.manager.read().unwrap();
    crate::block_on(crate::convert_async_panics(|| async {
      manager
        .internal_transfer(
          self.from_account_id.clone(),
          self.to_account_id.clone(),
          self.amount,
        )
        .await
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

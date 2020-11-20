use std::sync::{Arc, Mutex};

use iota_wallet::{
  account::SyncedAccount,
  message::{Message, Transfer},
  WalletError,
};
use neon::prelude::*;

pub struct SendTask {
  pub synced: Arc<Mutex<SyncedAccount>>,
  pub transfer: Transfer,
}

impl Task for SendTask {
  type Output = Message;
  type Error = WalletError;
  type JsEvent = JsValue;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    let synced = self.synced.lock().unwrap();
    crate::block_on(synced.transfer(self.transfer.clone()))
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

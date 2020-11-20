use std::sync::Arc;

use iota_wallet::{
  account::AccountIdentifier, account_manager::AccountManager, message::Message, WalletError,
};
use neon::prelude::*;

pub struct InternalTransferTask {
  pub manager: Arc<AccountManager>,
  pub from_account_id: AccountIdentifier,
  pub to_account_id: AccountIdentifier,
  pub amount: u64,
}

impl Task for InternalTransferTask {
  type Output = Message;
  type Error = WalletError;
  type JsEvent = JsValue;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    crate::block_on(crate::convert_async_panics(|| async {
      self
        .manager
        .internal_transfer(self.from_account_id, self.to_account_id, self.amount)
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

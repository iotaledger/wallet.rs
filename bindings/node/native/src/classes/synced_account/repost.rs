use std::sync::{Arc, RwLock};

use iota_wallet::{
  account::SyncedAccount,
  message::{Message, MessageId},
  WalletError,
};
use neon::prelude::*;

pub enum RepostAction {
  Retry,
  Reattach,
  Promote,
}

pub struct RepostTask {
  pub synced: Arc<RwLock<SyncedAccount>>,
  pub message_id: MessageId,
  pub action: RepostAction,
}

impl Task for RepostTask {
  type Output = Message;
  type Error = WalletError;
  type JsEvent = JsValue;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    let synced = self.synced.read().unwrap();
    crate::block_on(crate::convert_async_panics(|| async {
      match self.action {
        RepostAction::Retry => synced.retry(&self.message_id).await,
        RepostAction::Reattach => synced.reattach(&self.message_id).await,
        RepostAction::Promote => synced.promote(&self.message_id).await,
      }
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

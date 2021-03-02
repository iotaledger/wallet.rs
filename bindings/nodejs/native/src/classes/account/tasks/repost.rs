// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    message::{Message, MessageId},
    Error,
};
use neon::prelude::*;

pub enum RepostAction {
    Retry,
    Reattach,
    Promote,
}

pub struct RepostTask {
    pub account_id: String,
    pub message_id: MessageId,
    pub action: RepostAction,
}

impl Task for RepostTask {
    type Output = Message;
    type Error = Error;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        crate::block_on(crate::convert_async_panics(|| async {
            let account = crate::get_account(&self.account_id).await;
            let message = match self.action {
                RepostAction::Retry => account.retry(&self.message_id).await?,
                RepostAction::Reattach => account.reattach(&self.message_id).await?,
                RepostAction::Promote => account.promote(&self.message_id).await?,
            };

            Ok(message)
        }))
    }

    fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match value {
            Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}

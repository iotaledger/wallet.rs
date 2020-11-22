use std::sync::{Arc, Mutex};

use iota_wallet::{
  account::{Account, SyncedAccount},
  WalletError,
};
use neon::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct SyncOptions {
  #[serde(rename = "addressIndex")]
  address_index: Option<usize>,
  #[serde(rename = "gapLimit")]
  gap_limit: Option<usize>,
  #[serde(rename = "skipPersistance")]
  skip_persistance: Option<bool>,
}

pub struct SyncTask {
  pub account: Arc<Mutex<Account>>,
  pub options: SyncOptions,
}

impl Task for SyncTask {
  type Output = SyncedAccount;
  type Error = WalletError;
  type JsEvent = JsValue;

  fn perform(&self) -> Result<Self::Output, Self::Error> {
    let mut acc = self.account.lock().unwrap();
    let mut synchronizer = acc.sync();
    if let Some(address_index) = self.options.address_index {
      synchronizer = synchronizer.address_index(address_index);
    }
    if let Some(gap_limit) = self.options.gap_limit {
      synchronizer = synchronizer.gap_limit(gap_limit);
    }
    if let Some(skip_persistance) = self.options.skip_persistance {
      if skip_persistance {
        synchronizer = synchronizer.skip_persistance();
      }
    }
    crate::block_on(crate::convert_async_panics(|| async {
      synchronizer.execute().await
    }))
  }

  fn complete(
    self,
    mut cx: TaskContext,
    value: Result<Self::Output, Self::Error>,
  ) -> JsResult<Self::JsEvent> {
    match value {
      Ok(val) => {
        let synced = serde_json::to_string(&val).unwrap();
        let synced = cx.string(synced);
        Ok(crate::JsSyncedAccount::new(&mut cx, vec![synced])?.upcast())
      }
      Err(e) => cx.throw_error(e.to_string()),
    }
  }
}

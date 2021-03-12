// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unnecessary_wraps)]

use std::{
    any::Any,
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
};

use futures::{Future, FutureExt};
use iota::common::logger::{logger_init, LoggerConfigBuilder};
use iota_wallet::{
    account::{AccountHandle, SyncedAccount},
    Error,
};
use neon::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{runtime::Runtime, sync::RwLock};

mod classes;
use classes::*;
pub(crate) mod types;

type AccountInstanceMap = Arc<RwLock<HashMap<String, AccountHandle>>>;
type SyncedAccountHandle = Arc<RwLock<SyncedAccount>>;
type SyncedAccountInstanceMap = Arc<RwLock<HashMap<String, SyncedAccountHandle>>>;

/// Gets the account instances map.
fn account_instances() -> &'static AccountInstanceMap {
    static INSTANCES: Lazy<AccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) async fn get_account(id: &str) -> AccountHandle {
    account_instances()
        .read()
        .await
        .get(id)
        .expect("account dropped or not initialised")
        .clone()
}

pub(crate) async fn store_account(account_handle: AccountHandle) -> String {
    let handle = account_handle.clone();
    let id = handle.id().await;

    account_instances().write().await.insert(id.clone(), account_handle);

    id
}

/// Gets the synced account instances map.
fn synced_account_instances() -> &'static SyncedAccountInstanceMap {
    static INSTANCES: Lazy<SyncedAccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

#[allow(dead_code)]
pub(crate) async fn get_synced_account(id: &str) -> SyncedAccountHandle {
    synced_account_instances()
        .read()
        .await
        .get(id)
        .expect("synced account dropped or not initialised")
        .clone()
}

pub(crate) async fn store_synced_account(synced_account: SyncedAccount) -> String {
    let mut map = synced_account_instances().write().await;
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(10)
        .collect();
    map.insert(id.clone(), Arc::new(RwLock::new(synced_account)));
    id
}

pub(crate) async fn remove_synced_account(id: &str) {
    synced_account_instances().write().await.remove(id);
}

fn panic_to_response_message(panic: Box<dyn Any>) -> String {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = backtrace::Backtrace::new();
    format!("{}\n\n{:?}", msg, current_backtrace)
}

pub async fn convert_async_panics<T, F: Future<Output = Result<T, Error>>>(f: impl FnOnce() -> F) -> Result<T, Error> {
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(Error::Panic(panic_to_response_message(panic))),
    }
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

pub fn init_logger(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsString>(0)?.value();
    let config: LoggerConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    logger_init(config.finish()).expect("failed to init logger");
    Ok(cx.undefined())
}

// Export the class
register_module!(mut m, {
    m.export_function("initLogger", init_logger)?;
    m.export_class::<JsAccountManager>("AccountManager")?;
    m.export_class::<JsAccount>("Account")?;
    m.export_class::<JsSyncedAccount>("SyncedAccount")?;
    m.export_class::<JsEventListener>("EventListener")?;
    Ok(())
});

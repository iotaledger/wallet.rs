// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::Any,
    borrow::Cow,
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex, RwLock},
};

use futures::{Future, FutureExt};
use iota::common::logger::{logger_init, LoggerConfigBuilder};
use iota_wallet::{
    account::{AccountHandle, AccountIdentifier, SyncedAccount},
    Error,
};
use neon::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::runtime::Runtime;

mod classes;
use classes::*;

type AccountInstanceMap = Arc<RwLock<HashMap<AccountIdentifier, AccountHandle>>>;
type SyncedAccountHandle = Arc<RwLock<SyncedAccount>>;
type SyncedAccountInstanceMap = Arc<RwLock<HashMap<String, SyncedAccountHandle>>>;

/// Gets the account instances map.
fn account_instances() -> &'static AccountInstanceMap {
    static INSTANCES: Lazy<AccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) fn get_account(id: &AccountIdentifier) -> AccountHandle {
    account_instances()
        .read()
        .expect("failed to lock account instances: get_account()")
        .get(id)
        .expect("account dropped or not initialised")
        .clone()
}

pub(crate) fn store_account(account_handle: AccountHandle) -> AccountIdentifier {
    let handle = account_handle.clone();
    let id = block_on(async move { handle.id().await });

    account_instances()
        .write()
        .expect("failed to lock account instances: store_account()")
        .insert(id.clone(), account_handle);

    id
}

/// Gets the synced account instances map.
fn synced_account_instances() -> &'static SyncedAccountInstanceMap {
    static INSTANCES: Lazy<SyncedAccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) fn get_synced_account(id: &str) -> SyncedAccountHandle {
    synced_account_instances()
        .read()
        .expect("failed to lock synced account instances: get_synced_account()")
        .get(id)
        .expect("synced account dropped or not initialised")
        .clone()
}

pub(crate) fn store_synced_account(synced_account: SyncedAccount) -> String {
    let mut map = synced_account_instances()
        .write()
        .expect("failed to lock synced account instances: store_synced_account()");
    let id: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    map.insert(id.clone(), Arc::new(RwLock::new(synced_account)));
    id
}

pub(crate) fn remove_synced_account(id: &str) {
    synced_account_instances()
        .write()
        .expect("failed to lock synced account instances: remove_synced_account()")
        .remove(id);
}

fn panic_to_response_message(panic: Box<dyn Any>) -> Result<String, Error> {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = backtrace::Backtrace::new();
    Ok(format!("{}\n\n{:?}", msg, current_backtrace))
}

pub async fn convert_async_panics<T, F: Future<Output = Result<T, Error>>>(f: impl FnOnce() -> F) -> Result<T, Error> {
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(Error::Panic(panic_to_response_message(panic)?)),
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

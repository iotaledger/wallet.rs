// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::Any,
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use futures::{Future, FutureExt};
use iota_wallet::{
    account::{Account, AccountIdentifier},
    WalletError,
};
use neon::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::runtime::Runtime;

mod classes;
use classes::*;

type AccountInstanceMap = Arc<RwLock<HashMap<String, Arc<RwLock<Account>>>>>;

/// check if the account instance is loaded on the JS side (AccountInstanceMap) and update it by running a callback
fn mutate_account_if_exists<F: FnOnce(&mut Account) + Send + Sync + 'static>(account_id: &AccountIdentifier, cb: F) {
    let account_id = account_id.clone();
    thread::spawn(move || {
        let map = instances()
            .read()
            .expect("failed to lock read on account instances: mutate_account_if_exists()");

        for account in map.values() {
            let account_ = account.read().unwrap();
            if account_.id() == &account_id {
                std::mem::drop(account_);
                let mut account = account.write().unwrap();
                cb(&mut account);
                break;
            }
        }
    });
}

/// Gets the account instances map.
fn instances() -> &'static AccountInstanceMap {
    static INSTANCES: Lazy<AccountInstanceMap> = Lazy::new(|| {
        iota_wallet::event::on_balance_change(|event| {
            let address = event.cloned_address();
            let balance = *event.balance();
            mutate_account_if_exists(event.account_id(), move |account| {
                let addresses = account.addresses_mut();
                if let Some(address) = addresses.iter_mut().find(|a| a == &&address) {
                    address.set_balance(balance);
                }
            });
        });
        iota_wallet::event::on_new_transaction(|event| {
            let message = event.cloned_message();
            mutate_account_if_exists(event.account_id(), move |account| {
                account.append_messages(vec![message]);
            });
        });
        iota_wallet::event::on_confirmation_state_change(|event| {
            let message = event.cloned_message();
            let confirmed = *event.confirmed();
            mutate_account_if_exists(event.account_id(), move |account| {
                if let Some(message) = account.messages_mut().iter_mut().find(|m| m == &&message) {
                    message.set_confirmed(confirmed);
                }
            });
        });
        iota_wallet::event::on_reattachment(|event| {
            let message = event.cloned_message();
            mutate_account_if_exists(event.account_id(), move |account| {
                account.append_messages(vec![message]);
            });
        });
        iota_wallet::event::on_broadcast(|event| {
            let message = event.cloned_message();
            mutate_account_if_exists(event.account_id(), move |account| {
                if let Some(message) = account.messages_mut().iter_mut().find(|m| m == &&message) {
                    message.set_broadcasted(true);
                }
            });
        });
        Default::default()
    });
    &INSTANCES
}

pub(crate) fn get_account(id: &str) -> Arc<RwLock<Account>> {
    let map = instances()
        .read()
        .expect("failed to lock account instances: get_account()");
    map.get(id).expect("account dropped or not initialised").clone()
}

pub(crate) fn store_account(account: Account) -> String {
    let mut map = instances()
        .write()
        .expect("failed to lock account instances: store_account()");
    let id: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    map.insert(id.clone(), Arc::new(RwLock::new(account)));
    id
}

pub(crate) fn update_account(id: &str, account: Account) {
    let mut map = instances()
        .write()
        .expect("failed to lock account instances: store_account()");
    map.insert(id.to_string(), Arc::new(RwLock::new(account)));
}

pub(crate) fn remove_account(id: &str) {
    let mut map = instances()
        .write()
        .expect("failed to lock account instances: remove_account()");
    map.remove(id);
}

fn panic_to_response_message(panic: Box<dyn Any>) -> Result<String, WalletError> {
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

pub async fn convert_async_panics<T, F: Future<Output = Result<T, WalletError>>>(
    f: impl FnOnce() -> F,
) -> Result<T, WalletError> {
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(WalletError::UnknownError(panic_to_response_message(panic)?)),
    }
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

// Export the class
register_module!(mut m, {
    m.export_class::<JsAccountManager>("AccountManager")?;
    m.export_class::<JsAccount>("Account")?;
    m.export_class::<JsSyncedAccount>("SyncedAccount")?;
    m.export_class::<JsEventListener>("EventListener")?;
    Ok(())
});

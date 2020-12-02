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

use std::any::Any;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex, RwLock};

use futures::{Future, FutureExt};
use iota_wallet::{account::Account, address::Address, message::Message, WalletError};
use neon::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::runtime::Runtime;

mod classes;
use classes::*;

type AccountInstanceMap = Arc<RwLock<HashMap<String, Arc<RwLock<Account>>>>>;

fn mutate_account_if_exists<
    F: FnOnce(&Account, &mut Vec<Address>, &mut Vec<Message>) + Send + Sync,
>(
    account_id: &String,
    cb: F,
) {
    let map = instances()
        .read()
        .expect("failed to lock read on account instances: mutate_account_if_exists()");
    let mut found_account = None;
    for (instance_id, account) in map.iter() {
        let account_ = account.read().unwrap();
        if account_.id() == account_id {
            std::mem::drop(account_);
            let mut account = account.write().unwrap();
            let mut addresses: Vec<Address> = account.addresses().to_vec();
            let mut messages: Vec<Message> = account.messages().to_vec();
            cb(&account, &mut addresses, &mut messages);
            account.set_addresses(addresses);
            account.set_messages(messages);
            found_account = Some((instance_id.clone(), (*account).clone()));
            break;
        }
    }

    if let Some((id, account)) = found_account {
        std::mem::drop(map);
        update_account(&id, account);
    }
}

/// Gets the account instances map.
fn instances() -> &'static AccountInstanceMap {
    static INSTANCES: Lazy<AccountInstanceMap> = Lazy::new(|| {
        iota_wallet::event::on_balance_change(|event| {
            mutate_account_if_exists(event.account_id(), |_, addresses, _| {
                if let Some(address) = addresses.iter_mut().find(|a| a == event.address()) {
                    address.set_balance(*event.balance());
                }
            });
        });
        iota_wallet::event::on_new_transaction(|event| {
            mutate_account_if_exists(event.account_id(), |_, _, messages| {
                messages.push(event.cloned_message());
            });
        });
        iota_wallet::event::on_confirmation_state_change(|event| {
            mutate_account_if_exists(event.account_id(), |_, _, messages| {
                if let Some(message) = messages.iter_mut().find(|m| m == event.message()) {
                    message.set_confirmed(*event.confirmed());
                }
            });
        });
        iota_wallet::event::on_reattachment(|event| {
            mutate_account_if_exists(event.account_id(), |_, _, messages| {
                messages.push(event.cloned_message());
            });
        });
        iota_wallet::event::on_broadcast(|event| {
            mutate_account_if_exists(event.account_id(), |_, _, messages| {
                if let Some(message) = messages.iter_mut().find(|m| m == event.message()) {
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
    map.get(id)
        .expect("account dropped or not initialised")
        .clone()
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

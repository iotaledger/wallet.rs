use std::sync::{Arc, Mutex};

use iota_wallet::{
    account::{Account, SyncedAccount},
    account_manager::AccountManager,
    address::{parse as parse_address, Address},
    client::ClientOptions,
    message::Message,
    message::Transfer,
    DateTime, Utc,
};
use neon::prelude::*;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use tokio::runtime::Runtime;

mod tasks;
use tasks::send;
use tasks::sync;

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

#[derive(Deserialize)]
pub struct AccountToCreate {
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptions,
    pub mnemonic: Option<String>,
    pub alias: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

pub struct AccountWrapper(Arc<Mutex<Account>>);
pub struct SyncedAccountWrapper(Arc<Mutex<SyncedAccount>>);

declare_types! {
    pub class JsSyncedAccount for SyncedAccountWrapper {
        init(mut cx) {
            let synced = cx.argument::<JsValue>(0)?;
            let synced: SyncedAccount = neon_serde::from_value(&mut cx, synced)?;
            Ok(SyncedAccountWrapper(Arc::new(Mutex::new(synced))))
        }

        method send(mut cx) {
            let address = cx.argument::<JsString>(0)?.value();
            let amount = cx.argument::<JsNumber>(1)?.value() as u64;
            let cb = cx.argument::<JsFunction>(2)?;

            let transfer = Transfer::new(parse_address(address).expect("invalid address format"), amount);

            let this = cx.this();
            let synced = cx.borrow(&this, |r| r.0.clone());
            let task = send::SendTask {
                synced,
                transfer,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }

    pub class JsAccount for AccountWrapper {
        init(mut cx) {
            let account = cx.argument::<JsValue>(0)?;
            let account: Account = neon_serde::from_value(&mut cx, account)?;
            Ok(AccountWrapper(Arc::new(Mutex::new(account))))
        }

        method availableBalance(mut cx) {
            let balance = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.available_balance()
            };
            Ok(cx.number(balance as f64).upcast())
        }

        method totalBalance(mut cx) {
            let balance = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.total_balance()
            };
            Ok(cx.number(balance as f64).upcast())
        }

        method listMessages(mut cx) {
            let count = match cx.argument_opt(0) {
                Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut cx)?.value() as usize,
                None => 0,
            };
            let from = match cx.argument_opt(1) {
                Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut cx)?.value() as usize,
                None => 0,
            };
            let filter = match cx.argument_opt(0) {
                Some(arg) => {
                    let type_ = arg.downcast::<JsValue>().or_throw(&mut cx)?;
                    neon_serde::from_value(&mut cx, type_)?
                },
                None => None,
            };

            let messages: Vec<Message> = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.list_messages(count, from, filter).into_iter().cloned().collect()
            };

            let js_array = JsArray::new(&mut cx, messages.len() as u32);
            for (index, message) in messages.iter().enumerate() {
                let value = neon_serde::to_value(&mut cx, &message)?;
                js_array.set(&mut cx, index as u32, value)?;
            }

            Ok(js_array.upcast())
        }

        method listAddresses(mut cx) {
            let unspent = match cx.argument_opt(0) {
                Some(arg) => arg.downcast::<JsBoolean>().or_throw(&mut cx)?.value(),
                None => false,
            };

            let addresses: Vec<Address> = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.list_addresses(unspent).into_iter().cloned().collect()
            };

            let js_array = JsArray::new(&mut cx, addresses.len() as u32);
            for (index, address) in addresses.iter().enumerate() {
                let value = neon_serde::to_value(&mut cx, &address)?;
                js_array.set(&mut cx, index as u32, value)?;
            }

            Ok(js_array.upcast())
        }

        method sync(mut cx) {
            let options = match cx.argument_opt(0) {
                Some(arg) => {
                    let options = arg.downcast::<JsValue>().or_throw(&mut cx)?;
                    neon_serde::from_value(&mut cx, options)?
                }
                None => Default::default(),
            };
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let account = cx.borrow(&this, |r| r.0.clone());
            let task = sync::SyncTask {
                account,
                options,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }

    pub class JsAccountManager for AccountManager {
        init(mut cx) {
            let storage_path = match cx.argument_opt(0) {
                Some(arg) => {
                    Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value())
                }
                None => None,
            };
            let manager = match storage_path {
                Some(p) => AccountManager::with_storage_path(p).unwrap(),
                None => AccountManager::new().unwrap(),
            };
            Ok(manager)
        }

        method setStrongholdPassword(mut cx) {
            let password = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let manager = this.borrow(&guard);
                manager.set_stronghold_password(password).unwrap();
            }
            Ok(cx.undefined().upcast())
        }

        method createAccount(mut cx) {
            let account = {
                let account_to_create = cx.argument::<JsValue>(0)?;
                let account_to_create: AccountToCreate = neon_serde::from_value(&mut cx, account_to_create)?;
                let this = cx.this();
                let guard = cx.lock();
                let manager = this.borrow(&guard);

                let mut builder = manager
                    .create_account(account_to_create.client_options.clone());
                if let Some(mnemonic) = &account_to_create.mnemonic {
                    builder = builder.mnemonic(mnemonic);
                }
                if let Some(alias) = &account_to_create.alias {
                    builder = builder.alias(alias);
                }
                if let Some(created_at) = &account_to_create.created_at {
                    builder = builder.created_at(
                        created_at
                        .parse::<DateTime<Utc>>()
                        .unwrap(),
                    );
                }
                builder.initialise().unwrap()
            };
            let account = neon_serde::to_value(&mut cx, &account)?;

            Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
        }
    }
}
// Export the class
register_module!(mut m, {
    m.export_class::<JsAccountManager>("AccountManager")?;
    m.export_class::<JsAccount>("Account")?;
    Ok(())
});

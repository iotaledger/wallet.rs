use std::sync::{Arc, Mutex};

use iota_wallet::{
    account::Account, account_manager::AccountManager, client::ClientOptions, DateTime, Utc,
};
use neon::prelude::*;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use tokio::runtime::Runtime;

mod sync;

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

declare_types! {
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

        method sync(mut cx) {
            let options = cx.argument::<JsValue>(0)?;
            let options: sync::SyncOptions = neon_serde::from_value(&mut cx, options)?;
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

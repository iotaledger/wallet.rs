use super::JsAccount;
use std::convert::TryInto;
use std::sync::Arc;

use iota_wallet::{
    account::AccountIdentifier, account_manager::AccountManager, client::ClientOptions, DateTime,
    Utc,
};
use neon::prelude::*;
use serde::Deserialize;

mod internal_transfer;
mod sync;

#[derive(Deserialize)]
pub struct AccountToCreate {
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptions,
    pub mnemonic: Option<String>,
    pub alias: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

fn js_array_to_acount_id(
    cx: &mut CallContext<'_, JsAccountManager>,
    value: Handle<JsValue>,
) -> NeonResult<AccountIdentifier> {
    match value.downcast::<JsArray>() {
        Ok(js_array) => {
            let vec: Vec<Handle<JsValue>> = js_array.to_vec(cx)?;
            let mut id = vec![];
            for value in vec {
                let byte: JsNumber = *value.downcast_or_throw(cx)?;
                id.push(byte.value() as u8);
            }
            let id: [u8; 32] = id
                .try_into()
                .expect("account id must have exactly 32 bytes");
            Ok(id.into())
        }
        Err(_) => {
            let index: JsNumber = *value.downcast_or_throw(cx)?;
            Ok((index.value() as u64).into())
        }
    }
}

pub struct AccountManagerWrapper(Arc<AccountManager>);

declare_types! {
    pub class JsAccountManager for AccountManagerWrapper {
        init(mut cx) {
            let storage_path = match cx.argument_opt(0) {
                Some(arg) => {
                    Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value())
                }
                None => None,
            };
            let manager = match storage_path {
                Some(p) => AccountManager::with_storage_path(p),
                None => AccountManager::new(),
            };
            Ok(AccountManagerWrapper(Arc::new(manager.expect("error initializing account manager"))))
        }

        method setStrongholdPassword(mut cx) {
            let password = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;
                manager.set_stronghold_password(password).expect("error setting stronghold password");
            }
            Ok(cx.undefined().upcast())
        }

        method createAccount(mut cx) {
            let account = {
                let account_to_create = cx.argument::<JsValue>(0)?;
                let account_to_create: AccountToCreate = neon_serde::from_value(&mut cx, account_to_create)?;
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;

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
                        .expect("invalid account created at format"),
                    );
                }
                builder.initialise().expect("error creating account")
            };
            let account = serde_json::to_string(&account).unwrap();
            let account = cx.string(account);

            Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
        }

        method getAccount(mut cx) {
            let id = cx.argument::<JsValue>(0)?;
            let id = js_array_to_acount_id(&mut cx, id)?;
            let account = {
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;
                manager.get_account(id)
            };
            match account {
                Ok(acc) => {
                    let account = serde_json::to_string(&acc).unwrap();
                    let account = cx.string(account);
                    Ok(JsAccount::new(&mut cx, vec![account])?.upcast())
                },
                Err(_) => Ok(cx.undefined().upcast())
            }
        }

        method removeAccount(mut cx) {
            let id = cx.argument::<JsValue>(0)?;
            let id = js_array_to_acount_id(&mut cx, id)?;
            {
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;
                manager.remove_account(id).expect("error removing account")
            };
            Ok(cx.undefined().upcast())
        }

        method syncAccounts(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();
            let manager = cx.borrow(&this, |r| r.0.clone());
            let task = sync::SyncTask {
                manager,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method internalTransfer(mut cx) {
            let from_account_id = cx.argument::<JsValue>(0)?;
            let from_account_id = js_array_to_acount_id(&mut cx, from_account_id)?;
            let to_account_id = cx.argument::<JsValue>(1)?;
            let to_account_id = js_array_to_acount_id(&mut cx, to_account_id)?;
            let amount = cx.argument::<JsNumber>(2)?.value() as u64;
            let cb = cx.argument::<JsFunction>(3)?;

            let this = cx.this();
            let manager = cx.borrow(&this, |r| r.0.clone());
            let task = internal_transfer::InternalTransferTask {
                manager,
                from_account_id,
                to_account_id,
                amount,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method backup(mut cx) {
            let backup_path = cx.argument::<JsString>(0)?.value();
            let destination = {
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;
                manager.backup(backup_path).expect("error performing backup").display().to_string()
            };
            Ok(cx.string(destination).upcast())
        }

        method importAccounts(mut cx) {
            let source = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let manager = &this.borrow(&guard).0;
                manager.import_accounts(source).expect("error importing accounts");
            };
            Ok(cx.undefined().upcast())
        }
    }
}

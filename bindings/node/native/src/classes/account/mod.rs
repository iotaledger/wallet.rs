// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_wallet::{
    account::{Account, AccountIdentifier},
    address::Address,
    message::{Message, MessageId},
};
use neon::prelude::*;

mod sync;

pub struct AccountWrapper(pub String);

impl Drop for AccountWrapper {
    fn drop(&mut self) {
        crate::remove_account(&self.0);
    }
}

declare_types! {
    pub class JsAccount for AccountWrapper {
        init(mut cx) {
            let account = cx.argument::<JsString>(0)?.value();
            let account: Account = serde_json::from_str(&account).expect("invalid account JSON");
            let id = crate::store_account(account);
            Ok(AccountWrapper(id))
        }

        method id(mut cx) {
            let id = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.id().clone()
            };

            match id {
                AccountIdentifier::Id(id) => Ok(cx.string(id).upcast()),
                AccountIdentifier::Index(index) => Ok(cx.number(index as f64).upcast())
            }
        }

        method index(mut cx) {
            let index = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                *account.index()
            };

            Ok(cx.number(index as f64).upcast())
        }

        method alias(mut cx) {
            let alias = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.alias().clone()
            };

            Ok(cx.string(alias).upcast())
        }

        method availableBalance(mut cx) {
            let balance = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.available_balance()
            };
            Ok(cx.number(balance as f64).upcast())
        }

        method totalBalance(mut cx) {
            let balance = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
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
            let filter = match cx.argument_opt(2) {
                Some(arg) => {
                    let type_ = arg.downcast::<JsValue>().or_throw(&mut cx)?;
                    neon_serde::from_value(&mut cx, type_)?
                },
                None => None,
            };

            let messages: Vec<Message> = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
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
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.list_addresses(unspent).into_iter().cloned().collect()
            };

            let js_array = JsArray::new(&mut cx, addresses.len() as u32);
            for (index, address) in addresses.iter().enumerate() {
                let value = neon_serde::to_value(&mut cx, &address)?;
                js_array.set(&mut cx, index as u32, value)?;
            }

            Ok(js_array.upcast())
        }

        method setAlias(mut cx) {
            let alias = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let mut account = account.write().unwrap();
                account.set_alias(alias);
                account.save_pending_changes().expect("failed to save account");
            }
            Ok(cx.undefined().upcast())
        }

        method setClientOptions(mut cx) {
            let client_options = cx.argument::<JsValue>(0)?;
            let client_options = neon_serde::from_value(&mut cx, client_options)?;
            {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let mut account = account.write().unwrap();
                account.set_client_options(client_options);
                account.save_pending_changes().expect("failed to save account");
            }
            Ok(cx.undefined().upcast())
        }

        method getMessage(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let message = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.get_message(&message_id).cloned()
            };
            match message {
                Some(m) => Ok(neon_serde::to_value(&mut cx, &m)?),
                None => Ok(cx.undefined().upcast())
            }
        }

        method generateAddress(mut cx) {
            let address = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let mut account = account.write().unwrap();
                account.generate_address().expect("error generating address")
            };
            Ok(neon_serde::to_value(&mut cx, &address)?)
        }

        method latestAddress(mut cx) {
            let address = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                let account = crate::get_account(id);
                let account = account.read().unwrap();
                account.latest_address().cloned()
            };
            match address {
                Some(a) => Ok(neon_serde::to_value(&mut cx, &a)?),
                None => Ok(cx.undefined().upcast())
            }
        }

        method sync(mut cx) {
            let (options, cb) = match cx.argument_opt(1) {
                Some(arg) => {
                    let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
                    let options = cx.argument::<JsValue>(0)?;
                    let options = neon_serde::from_value(&mut cx, options)?;
                    (options, cb)
                }
                None => (Default::default(), cx.argument::<JsFunction>(0)?),
            };

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = sync::SyncTask {
                account_id,
                options,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

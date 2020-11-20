use std::str::FromStr;
use std::sync::{Arc, Mutex};

use iota_wallet::{
    account::Account,
    address::Address,
    message::{Message, MessageId},
};
use neon::prelude::*;

mod sync;

pub struct AccountWrapper(Arc<Mutex<Account>>);

declare_types! {
    pub class JsAccount for AccountWrapper {
        init(mut cx) {
            let account = cx.argument::<JsValue>(0)?;
            let account: Account = neon_serde::from_value(&mut cx, account)?;
            Ok(AccountWrapper(Arc::new(Mutex::new(account))))
        }

        method id(mut cx) {
            let id = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                *account.id()
            };

            let js_array = JsArray::new(&mut cx, id.len() as u32);
            for (index, b) in id.iter().enumerate() {
                let value = cx.number(*b);
                js_array.set(&mut cx, index as u32, value)?;
            }

            Ok(js_array.upcast())
        }

        method index(mut cx) {
            let index = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                *account.index()
            };

            Ok(cx.number(index as f64).upcast())
        }

        method alias(mut cx) {
            let alias = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.alias().clone()
            };

            Ok(cx.string(alias).upcast())
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

        method setAlias(mut cx) {
            let alias = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let mut account = ref_.lock().unwrap();
                account.set_alias(alias).expect("error updating account alias");
            }
            Ok(cx.undefined().upcast())
        }

        method getMessage(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let message = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
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
                let ref_ = &this.borrow(&guard).0;
                let mut account = ref_.lock().unwrap();
                account.generate_address().expect("error generating address")
            };
            Ok(neon_serde::to_value(&mut cx, &address)?)
        }

        method latestAddress(mut cx) {
            let address = {
                let this = cx.this();
                let guard = cx.lock();
                let ref_ = &this.borrow(&guard).0;
                let account = ref_.lock().unwrap();
                account.latest_address().cloned()
            };
            match address {
                Some(a) => Ok(neon_serde::to_value(&mut cx, &a)?),
                None => Ok(cx.undefined().upcast())
            }
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
}

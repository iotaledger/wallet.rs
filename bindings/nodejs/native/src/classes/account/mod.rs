// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::ClientOptionsDto;

use std::{num::NonZeroU64, str::FromStr};

use iota_wallet::{
    address::parse as parse_address,
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
};
use neon::prelude::*;
use serde::Deserialize;

mod synced_account;
mod tasks;

pub use synced_account::*;

#[derive(Deserialize)]
struct IndexationDto {
    index: Vec<u8>,
    data: Option<Vec<u8>>,
}

#[derive(Default, Deserialize)]
struct TransferOptions {
    #[serde(rename = "remainderValueStrategy", default)]
    remainder_value_strategy: RemainderValueStrategy,
    indexation: Option<IndexationDto>,
    #[serde(rename = "skipSync", default)]
    skip_sync: bool,
}

pub struct AccountWrapper(pub String);

declare_types! {
    pub class JsAccount for AccountWrapper {
        init(mut cx) {
            let account_id = cx.argument::<JsString>(0)?.value();
            Ok(AccountWrapper(serde_json::from_str(&account_id).expect("invalid account identifier")))
        }

        method id(mut cx) {
            let id = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                id.clone()
            };

            Ok(cx.string(id).upcast())
        }

        method index(mut cx) {
            let index = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.index().await
                })
            };

            Ok(cx.number(index as f64).upcast())
        }

        method alias(mut cx) {
            let alias = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.alias().await
                })
            };

            Ok(cx.string(alias).upcast())
        }

        method balance(mut cx) {
            let balance = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.balance().await
                }).expect("failed to get account balance")
            };
            Ok(neon_serde::to_value(&mut cx, &balance)?.upcast())
        }

        method getNodeInfo(mut cx) {
            let url: Option<String> = match cx.argument_opt(1) {
                Some(_arg) => {
                    Some(cx.argument::<JsString>(0)?.value())
                },
                None => Default::default(),
            };

            let cb = cx.argument::<JsFunction>(cx.len()-1)?;
            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::NodeInfoTask {
                account_id,
                url,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method messageCount(mut cx) {
            let message_type = match cx.argument_opt(0) {
                Some(arg) => {
                    let type_ = arg.downcast::<JsValue>().or_throw(&mut cx)?;
                    neon_serde::from_value(&mut cx, type_)?
                },
                None => None,
            };
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let count = account.list_messages(0, 0, message_type).await.expect("failed to list messages").iter().len();
                Ok(cx.number(count as f64).upcast())
            })
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

            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let messages = account.list_messages(count, from, filter).await.expect("failed to list messages");

                let js_array = JsArray::new(&mut cx, messages.len() as u32);
                for (index, message) in messages.iter().enumerate() {
                    let value = neon_serde::to_value(&mut cx, &message)?;
                    js_array.set(&mut cx, index as u32, value)?;
                }

                Ok(js_array.upcast())
            })
        }

        method listAddresses(mut cx) {
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let addresses = account.addresses();

                let js_array = JsArray::new(&mut cx, addresses.len() as u32);
                for (index, address) in addresses.iter().enumerate() {
                    let value = neon_serde::to_value(&mut cx, &address)?;
                    js_array.set(&mut cx, index as u32, value)?;
                }

                Ok(js_array.upcast())
            })
        }

        method listSpentAddresses(mut cx) {
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let addresses = account.list_spent_addresses().await.expect("failed to list addresses");

                let js_array = JsArray::new(&mut cx, addresses.len() as u32);
                for (index, address) in addresses.iter().enumerate() {
                    let value = neon_serde::to_value(&mut cx, &address)?;
                    js_array.set(&mut cx, index as u32, value)?;
                }

                Ok(js_array.upcast())
            })
        }

        method listUnspentAddresses(mut cx) {
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let addresses = account.list_unspent_addresses().await.expect("failed to list addresses");

                let js_array = JsArray::new(&mut cx, addresses.len() as u32);
                for (index, address) in addresses.iter().enumerate() {
                    let value = neon_serde::to_value(&mut cx, &address)?;
                    js_array.set(&mut cx, index as u32, value)?;
                }

                Ok(js_array.upcast())
            })
        }

        method setAlias(mut cx) {
            let alias = cx.argument::<JsString>(0)?.value();
            {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.set_alias(alias).await.expect("failed to update account alias");
                });
            }
            Ok(cx.undefined().upcast())
        }

        method setClientOptions(mut cx) {
            let client_options = cx.argument::<JsValue>(0)?;
            let client_options: ClientOptionsDto = neon_serde::from_value(&mut cx, client_options)?;
            {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.set_client_options(client_options.into()).await.expect("failed to update client options");
                });
            }
            Ok(cx.undefined().upcast())
        }

        method getMessage(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let message = account.get_message(&message_id).await;
                match message {
                    Some(m) => Ok(neon_serde::to_value(&mut cx, &m)?),
                    None => Ok(cx.undefined().upcast())
                }
            })
        }

        method getAddress(mut cx) {
            let address = parse_address(cx.argument::<JsString>(0)?.value()).expect("invalid address");
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let address = account.addresses().iter().find(|a| a.address() == &address);
                match address {
                    Some(a) => Ok(neon_serde::to_value(&mut cx, &a)?),
                    None => Ok(cx.undefined().upcast())
                }
            })
        }

        method generateAddress(mut cx) {
            let address = {
                let this = cx.this();
                let guard = cx.lock();
                let id = &this.borrow(&guard).0;
                crate::block_on(async move {
                    let account_handle = crate::get_account(id).await;
                    account_handle.generate_address().await.expect("error generating address")
                })
            };
            Ok(neon_serde::to_value(&mut cx, &address)?)
        }

        method latestAddress(mut cx) {
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let account = account_handle.read().await;
                let address = account.latest_address();
                Ok(neon_serde::to_value(&mut cx, &address)?)
            })
        }

        method getUnusedAddress(mut cx) {
            let this = cx.this();
            let id = cx.borrow(&this, |r| r.0.clone());
            crate::block_on(async move {
                let account_handle = crate::get_account(&id).await;
                let address = account_handle.get_unused_address().await;
                Ok(neon_serde::to_value(&mut cx, &address)?)
            })
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
            let task = tasks::SyncTask {
                account_id,
                options,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method send(mut cx) {
            let address = cx.argument::<JsString>(0)?.value();
            let amount = cx.argument::<JsNumber>(1)?.value() as u64;
            let (options, cb) = match cx.argument_opt(3) {
                Some(arg) => {
                    let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
                    let options = cx.argument::<JsValue>(2)?;
                    let options = neon_serde::from_value(&mut cx, options)?;
                    (options, cb)
                }
                None => (TransferOptions::default(), cx.argument::<JsFunction>(2)?),
            };

            let mut transfer_builder = Transfer::builder(
                parse_address(address).expect("invalid address format"),
                NonZeroU64::new(amount).expect("amount can't be zero")
            ).with_remainder_value_strategy(options.remainder_value_strategy);
            if let Some(indexation) = options.indexation {
                transfer_builder = transfer_builder.with_indexation(
                    IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be empty")
                );
            }
            if options.skip_sync {
                transfer_builder = transfer_builder.with_skip_sync();
            }

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::SendTask {
                account_id,
                transfer: transfer_builder.finish(),
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method sendToMany(mut cx) {
            let js_arr_handle: Handle<JsArray> = cx.argument(0)?;
            let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;
            let mut outputs = Vec::new();

            for js_value in vec {
                let js_object = js_value.downcast::<JsObject>().unwrap();
                let address = js_object.get(&mut cx, "address")?.downcast::<JsString>().or_throw(&mut cx)?;
                let amount = js_object.get(&mut cx, "amount")?.downcast::<JsNumber>().or_throw(&mut cx)?;
                outputs.push(TransferOutput::new(
                    parse_address(address.value()).expect("invalid address format"),
                    NonZeroU64::new(amount.value() as u64).expect("amount can't be zero"),
                ));
            }

            let (options, cb) = match cx.argument_opt(2) {
                Some(arg) => {
                    let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
                    let options = cx.argument::<JsValue>(1)?;
                    let options = neon_serde::from_value(&mut cx, options)?;
                    (options, cb)
                }
                None => (TransferOptions::default(), cx.argument::<JsFunction>(1)?),
            };

            let mut transfer_builder = Transfer::builder_with_outputs(outputs).expect("Outputs must be less then 125")
                .with_remainder_value_strategy(options.remainder_value_strategy);
            if let Some(indexation) = options.indexation {
                transfer_builder = transfer_builder.with_indexation(
                    IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be empty")
                );
            }
            if options.skip_sync {
                transfer_builder = transfer_builder.with_skip_sync();
            }

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::SendTask {
                account_id,
                transfer: transfer_builder.finish(),
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method retry(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::RepostTask {
                account_id,
                message_id,
                action: tasks::RepostAction::Retry,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method reattach(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::RepostTask {
                account_id,
                message_id,
                action: tasks::RepostAction::Reattach,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method promote(mut cx) {
            let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id length");
            let cb = cx.argument::<JsFunction>(1)?;

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::RepostTask {
                account_id,
                message_id,
                action: tasks::RepostAction::Promote,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method consolidateOutputs(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::ConsolidateOutputsTask {
                account_id,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }

        method isLatestAddressUnused(mut cx) {
            let cb = cx.argument::<JsFunction>(0)?;

            let this = cx.this();
            let account_id = cx.borrow(&this, |r| r.0.clone());
            let task = tasks::IsLatestAddressUnusedTask {
                account_id,
            };
            task.schedule(cb);
            Ok(cx.undefined().upcast())
        }
    }
}

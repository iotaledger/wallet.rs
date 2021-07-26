// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::ClientOptionsDto;

use std::{num::NonZeroU64, str::FromStr};

use iota_wallet::{
    account,
    address::parse as parse_address,
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
};
use neon::prelude::*;
use serde::Deserialize;
use serde_json::StreamDeserializer;
use std::sync::{mpsc::channel, Arc};

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

#[derive(Deserialize, Default)]
pub struct SyncOptions {
    #[serde(rename = "addressIndex")]
    address_index: Option<usize>,
    #[serde(rename = "gapLimit")]
    gap_limit: Option<usize>,
}

#[derive(Deserialize, Default)]
pub struct NodeInfoOptions {
    name: Option<String>,
    password: Option<String>,
    jwt: Option<String>,
}

pub struct AccountWrapper {
    account_id: String,
    queue: EventQueue,
}
impl Finalize for AccountWrapper {}
// pub fn send_message(mut cx: FunctionContext) -> JsResult<JsUndefined> {
// let message = cx.argument::<JsString>(0)?;
// let message = message.value(&mut cx);
// let message_handler = Arc::clone(&&cx.argument::<JsBox<Arc<MessageHandler>>>(1)?);
// let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);
//
// crate::RUNTIME.spawn(async move {
// let (response, is_error) = message_handler.send_message(message).await;
// log::debug!("{:?}", response);
// message_handler.queue.send(move |mut cx| {
// let cb = callback.into_inner(&mut cx);
// let this = cx.undefined();
//
// let args = vec![
// if is_error {
// cx.string(response.clone()).upcast::<JsValue>()
// } else {
// cx.undefined().upcast::<JsValue>()
// },
// cx.string(response.clone()).upcast::<JsValue>(),
// ];
//
// cb.call(&mut cx, this, args)?;
//
// Ok(())
// });
// });
//
// Ok(cx.undefined())
// }
impl AccountWrapper {
    pub fn new(queue: EventQueue, account_id: String) -> Arc<Self> {
        Arc::new(Self { account_id, queue })
    }
}

pub fn account_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<AccountWrapper>>> {
    let account_id = cx.argument::<JsString>(0)?;
    let account_id = account_id.value(&mut cx);
    let queue = cx.queue();
    let account_wrapper = AccountWrapper::new(queue, account_id);

    Ok(cx.boxed(account_wrapper))
}

pub fn id(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);

    Ok(cx.string(account_wrapper.account_id.clone()))
}

pub fn index(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        let index = account_handle.index().await;
        let _ = sender.send(index);
    });

    Ok(cx.number(receiver.recv().unwrap() as f64))
}

pub fn alias(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        let alias = account_handle.alias().await;
        let _ = sender.send(alias);
    });

    Ok(cx.string(receiver.recv().unwrap()))
}

pub fn balance(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        let alias = account_handle.balance().await;
        let _ = sender.send(alias);
    });
    let balance = serde_json::to_string(&receiver.recv().unwrap()).unwrap();
    Ok(cx.string(balance))
}

pub fn get_node_info(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(cx.len() - 2)?);
    let callback = cx.argument::<JsFunction>(cx.len() - 1)?.root(&mut cx);
    let id = account_wrapper.account_id.clone();

    let url: Option<String> = match cx.argument_opt(0) {
        Some(arg) => match arg.downcast::<JsString, FunctionContext>(&mut cx) {
            Ok(url) => Some(url.value(&mut cx)),
            _ => None,
        },
        None => None,
    };

    let (jwt, auth) = match cx.argument_opt(1) {
        Some(arg) => match arg.downcast::<JsString, FunctionContext>(&mut cx) {
            Ok(options) => match serde_json::from_str::<NodeInfoOptions>(options.value(&mut cx).as_str()) {
                Ok(options) => {
                    let name_password = if options.name.is_some() && options.password.is_some() {
                        Some((options.name.unwrap(), options.password.unwrap()))
                    } else {
                        None
                    };
                    (options.jwt, name_password)
                }
                Err(_) => (None, None),
            },
            _ => (None, None),
        },
        None => (None, None),
    };

    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let account = account_handle.read().await;
        let node_info = match auth {
            Some((name, password)) => account
                .get_node_info(url.as_deref(), jwt.as_deref(), Some((&name, &password)))
                .await
                .expect("failed to get nodeinfo"),
            None => account
                .get_node_info(url.as_deref(), jwt.as_deref(), None)
                .await
                .expect("failed to get nodeinfo"),
        };

        account_wrapper.queue.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                cx.undefined().upcast::<JsValue>(),
                cx.string(serde_json::to_string(&node_info).unwrap()).upcast::<JsValue>()
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn message_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let message_type = match cx.argument_opt(0) {
        Some(arg) => {
            let type_ = arg.downcast::<JsString, FunctionContext>(&mut cx).or_throw(&mut cx)?;
            serde_json::from_str(type_.value(&mut cx).as_str()).unwrap()
        }
        None => None,
    };
    let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(1)?);
    let id = account_wrapper.account_id.clone();
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let account = account_handle.read().await;
        let count = account
            .list_messages(0, 0, message_type)
            .await
            .expect("failed to list messages")
            .iter()
            .len();
        let _ = sender.send(count);
    });

    Ok(cx.number(receiver.recv().unwrap() as f64))
}

// pub fn listMessages(mut cx: FunctionContext) -> JsResult<JsArray> {
//     let count = match cx.argument_opt(0) {
//         Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut cx)?.value() as usize,
//         None => 0,
//     };
//     let from = match cx.argument_opt(1) {
//         Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut cx)?.value() as usize,
//         None => 0,
//     };
//     let filter = match cx.argument_opt(2) {
//         Some(arg) => {
//             let type_ = arg.downcast::<JsString>().or_throw(&mut cx)?;
//             serde_json::from_str::<>(type_)?
//         },
//         None => None,
//     };

//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let messages = account.list_messages(count, from, filter).await.expect("failed to list messages");

//         let js_array = JsArray::new(&mut cx, messages.len() as u32);
//         for (index, message) in messages.iter().enumerate() {
//             let value =  serde_json::to_string(&message)?;
//             js_array.set(&mut cx, index as u32, value)?;
//         }

//         Ok(js_array.upcast())
//     })
// }

// pub fn list_addresses(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let addresses = account.addresses();

//         let js_array = JsArray::new(&mut cx, addresses.len() as u32);
//         for (index, address) in addresses.iter().enumerate() {
//             let value =  serde_json::to_string(&address)?;
//             js_array.set(&mut cx, index as u32, value)?;
//         }

//         Ok(js_array.upcast())
//     })
// }

// pub fn listSpentAddresses(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let addresses = account.list_spent_addresses().await.expect("failed to list addresses");

//         let js_array = JsArray::new(&mut cx, addresses.len() as u32);
//         for (index, address) in addresses.iter().enumerate() {
//             let value =  serde_json::to_string(&address)?;
//             js_array.set(&mut cx, index as u32, value)?;
//         }

//         Ok(js_array.upcast())
//     })
// }

// pub fn listUnspentAddresses(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let addresses = account.list_unspent_addresses().await.expect("failed to list addresses");

//         let js_array = JsArray::new(&mut cx, addresses.len() as u32);
//         for (index, address) in addresses.iter().enumerate() {
//             let value =  serde_json::to_string(&address)?;
//             js_array.set(&mut cx, index as u32, value)?;
//         }

//         Ok(js_array.upcast())
//     })
// }

// pub fn setAlias(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let alias = cx.argument::<JsString>(0)?.value();
//     {
//         let this = cx.this();
//         let guard = cx.lock();
//         let id = &this.borrow(&guard).0;
//         crate::RUNTIME.spawn(async move {
//             let account_handle = crate::get_account(id).await;
//             account_handle.set_alias(alias).await.expect("failed to update account alias");
//         });
//     }
//     Ok(cx.undefined().upcast())
// }

// pub fn setClientOptions(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let client_options = cx.argument::<JsString>(0)?;
//     let client_options: ClientOptionsDto = serde_json::from_str::<ClientOptionsDto>(client_options)?;
//     {
//         let this = cx.this();
//         let guard = cx.lock();
//         let id = &this.borrow(&guard).0;
//         crate::RUNTIME.spawn(async move {
//             let account_handle = crate::get_account(id).await;
//             account_handle.set_client_options(client_options.into()).await.expect("failed to update client options");
//         });
//     }
//     Ok(cx.undefined().upcast())
// }

// pub fn getMessage(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id
// length");     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let message = account.get_message(&message_id).await;
//         match message {
//             Some(m) => Ok( serde_json::to_string(&m)?),
//             None => Ok(cx.undefined().upcast())
//         }
//     })
// }

// pub fn getAddress(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let address = parse_address(cx.argument::<JsString>(0)?.value()).expect("invalid address");
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let address = account.addresses().iter().find(|a| a.address() == &address);
//         match address {
//             Some(a) => Ok( serde_json::to_string(&a)?),
//             None => Ok(cx.undefined().upcast())
//         }
//     })
// }

pub fn generate_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(wrapper.account_id.as_str()).await;
        let address = account_handle
            .generate_address()
            .await
            .expect("error generating address");
        let _ = sender.send(address);
    });
    let address = receiver.recv().unwrap();

    Ok(cx.string(serde_json::to_string(&address).unwrap()))
}

// pub fn generateAddresses(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let amount = cx.argument::<JsNumber>(0)?.value() as usize;
//     let address = {
//         let this = cx.this();
//         let guard = cx.lock();
//         let id = &this.borrow(&guard).0;
//         crate::RUNTIME.spawn(async move {
//             let account_handle = crate::get_account(id).await;
//             account_handle.generate_addresses(amount).await.expect("error generating address")
//         })
//     };
//     Ok( serde_json::to_string(&address)?)
// }

// pub fn latestAddress(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     let (sender, receiver) = channel();
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let account = account_handle.read().await;
//         let address = account.latest_address();
//         sender.send()
//     });

//     Ok(serde_json::to_string(&address)?)
// }

// pub fn getUnusedAddress(mut cx: FunctionContext) -> JsResult<JsString> {
//     let this = cx.this();
//     let id = cx.borrow(&this, |r| r.0.clone());
//     let (sender, receiver) = channel();
//     crate::RUNTIME.spawn(async move {
//         let account_handle = crate::get_account(&id).await;
//         let address = account_handle.get_unused_address().await;
//         sender.send(address);
//     });

//     Ok(serde_json::to_string(receiver.recv().unwrap())?)
// }

pub fn sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let (options, account_wrapper, callback) = match cx.argument_opt(2) {
        Some(arg) => {
            let cb = arg
                .downcast::<JsFunction, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .root(&mut cx);
            let options = cx.argument::<JsString>(0)?;
            let options = serde_json::from_str::<SyncOptions>(options.value(&mut cx).as_str()).unwrap();
            let account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(1)?);
            (options, account_wrapper, cb)
        }
        None => (
            Default::default(),
            Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?),
            cx.argument::<JsFunction>(1)?.root(&mut cx),
        ),
    };

    let id = account_wrapper.account_id.clone();
    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;
        let mut synchronizer = account.sync().await;
        if let Some(address_index) = options.address_index {
            synchronizer = synchronizer.address_index(address_index);
        }
        if let Some(gap_limit) = options.gap_limit {
            synchronizer = synchronizer.gap_limit(gap_limit);
        }
        let _synced_account = synchronizer.execute().await;

        account_wrapper.queue.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![cx.undefined().upcast::<JsValue>(), cx.string(id).upcast::<JsValue>()];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

// pub fn send(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let address = cx.argument::<JsString>(0)?.value();
//     let amount = cx.argument::<JsNumber>(1)?.value() as u64;
//     let (options, cb) = match cx.argument_opt(3) {
//         Some(arg) => {
//             let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
//             let options = cx.argument::<JsString>(2)?;
//             let options = serde_json::from_str::<>(options)?;
//             (options, cb)
//         }
//         None => (TransferOptions::default(), cx.argument::<JsFunction>(2)?),
//     };

//     let mut transfer_builder = Transfer::builder(
//         parse_address(address).expect("invalid address format"),
//         NonZeroU64::new(amount).expect("amount can't be zero")
//     ).with_remainder_value_strategy(options.remainder_value_strategy);
//     if let Some(indexation) = options.indexation {
//         transfer_builder = transfer_builder.with_indexation(
//             IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be
// empty")         );
//     }
//     if options.skip_sync {
//         transfer_builder = transfer_builder.with_skip_sync();
//     }

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::SendTask {
//         account_id,
//         transfer: transfer_builder.finish(),
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn sendToMany(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let js_arr_handle: Handle<JsArray> = cx.argument(0)?;
//     let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;
//     let mut outputs = Vec::new();

//     for js_value in vec {
//         let js_object = js_value.downcast::<JsObject>().unwrap();
//         let address = js_object.get(&mut cx, "address")?.downcast::<JsString>().or_throw(&mut cx)?;
//         let amount = js_object.get(&mut cx, "amount")?.downcast::<JsNumber>().or_throw(&mut cx)?;
//         outputs.push(TransferOutput::new(
//             parse_address(address.value()).expect("invalid address format"),
//             NonZeroU64::new(amount.value() as u64).expect("amount can't be zero"),
//         ));
//     }

//     let (options, cb) = match cx.argument_opt(2) {
//         Some(arg) => {
//             let cb = arg.downcast::<JsFunction>().or_throw(&mut cx)?;
//             let options = cx.argument::<JsString>(1)?;
//             let options = serde_json::from_str::<>(options)?;
//             (options, cb)
//         }
//         None => (TransferOptions::default(), cx.argument::<JsFunction>(1)?),
//     };

//     let mut transfer_builder = Transfer::builder_with_outputs(outputs).expect("Outputs must be less then 125")
//         .with_remainder_value_strategy(options.remainder_value_strategy);
//     if let Some(indexation) = options.indexation {
//         transfer_builder = transfer_builder.with_indexation(
//             IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default()).expect("index can't be
// empty")         );
//     }
//     if options.skip_sync {
//         transfer_builder = transfer_builder.with_skip_sync();
//     }

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::SendTask {
//         account_id,
//         transfer: transfer_builder.finish(),
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn retry(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id
// length");     let cb = cx.argument::<JsFunction>(1)?;

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::RepostTask {
//         account_id,
//         message_id,
//         action: tasks::RepostAction::Retry,
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn reattach(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id
// length");     let cb = cx.argument::<JsFunction>(1)?;

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::RepostTask {
//         account_id,
//         message_id,
//         action: tasks::RepostAction::Reattach,
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn promote(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let message_id = MessageId::from_str(cx.argument::<JsString>(0)?.value().as_str()).expect("invalid message id
// length");     let cb = cx.argument::<JsFunction>(1)?;

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::RepostTask {
//         account_id,
//         message_id,
//         action: tasks::RepostAction::Promote,
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn consolidateOutputs(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let cb = cx.argument::<JsFunction>(0)?;
//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::ConsolidateOutputsTask {
//         account_id,
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

// pub fn isLatestAddressUnused(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let cb = cx.argument::<JsFunction>(0)?;

//     let this = cx.this();
//     let account_id = cx.borrow(&this, |r| r.0.clone());
//     let task = tasks::IsLatestAddressUnusedTask {
//         account_id,
//     };
//     task.schedule(cb);
//     Ok(cx.undefined().upcast())
// }

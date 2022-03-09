// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::ClientOptionsDto;

use std::{num::NonZeroU64, str::FromStr};

use iota_wallet::{
    address::{parse as parse_address, OutputKind},
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
};
use neon::prelude::*;
use serde::Deserialize;
use std::sync::{mpsc::channel, Arc};

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
    #[serde(rename = "outputKind", default)]
    output_kind: Option<OutputKind>,
}

#[derive(Deserialize, Default)]
pub struct SyncOptions {
    #[serde(rename = "addressIndex")]
    pub address_index: Option<usize>,
    #[serde(rename = "gapLimit")]
    pub gap_limit: Option<usize>,
}

#[derive(Deserialize, Default)]
pub struct NodeInfoOptions {
    name: Option<String>,
    password: Option<String>,
    jwt: Option<String>,
}

pub struct AccountWrapper {
    pub account_id: String,
    pub channel: Channel,
}
impl Finalize for AccountWrapper {}

impl AccountWrapper {
    pub fn new(channel: Channel, account_id: String) -> Arc<Self> {
        Arc::new(Self { account_id, channel })
    }
}

pub fn account_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<AccountWrapper>>> {
    let account_id = cx.argument::<JsString>(0)?;
    let account_id = account_id.value(&mut cx);
    let channel = cx.channel();
    let account_wrapper = AccountWrapper::new(channel, account_id);

    Ok(cx.boxed(account_wrapper))
}

pub fn id(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );

    Ok(cx.string(account_wrapper.account_id.clone()))
}

pub fn index(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
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
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
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
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        let balance = account_handle.balance().await.expect("failed to get balance");
        let _ = sender.send(balance);
    });
    let balance = serde_json::to_string(&receiver.recv().unwrap()).unwrap();
    Ok(cx.string(balance))
}

pub fn get_node_info(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
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

        account_wrapper.channel.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                cx.undefined().upcast::<JsValue>(),
                cx.string(serde_json::to_string(&node_info).unwrap())
                    .upcast::<JsValue>(),
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
            let type_ = arg.downcast::<JsNumber, FunctionContext>(&mut cx).or_throw(&mut cx)?;
            serde_json::from_str(&type_.value(&mut cx).to_string()).unwrap()
        }
        None => None,
    };
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
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

pub fn list_messages(mut cx: FunctionContext) -> JsResult<JsArray> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );

    let count = match cx.argument_opt(0) {
        Some(arg) => arg
            .downcast::<JsNumber, FunctionContext>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx) as usize,
        None => 0,
    };
    let from = match cx.argument_opt(1) {
        Some(arg) => arg
            .downcast::<JsNumber, FunctionContext>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx) as usize,
        None => 0,
    };
    let filter = match cx.argument_opt(2) {
        Some(arg) => {
            let type_ = arg.downcast::<JsNumber, FunctionContext>(&mut cx).or_throw(&mut cx)?;
            serde_json::from_str(&type_.value(&mut cx).to_string()).unwrap()
        }
        None => None,
    };

    let id = account_wrapper.account_id.clone();
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let account = account_handle.read().await;
        let messages = account
            .list_messages(count, from, filter)
            .await
            .expect("failed to list messages");

        let mut result = vec![];
        for message in messages.iter() {
            result.push(serde_json::to_string(&message).unwrap());
        }

        let _ = sender.send(result);
    });

    let messages = receiver.recv().unwrap();
    let js_array = JsArray::new(&mut cx, messages.len() as u32);
    for (index, message) in messages.iter().enumerate() {
        let msg = cx.string(message);
        js_array.set(&mut cx, index as u32, msg)?;
    }

    Ok(js_array)
}

pub fn list_addresses(mut cx: FunctionContext) -> JsResult<JsArray> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let unspent = match cx.argument_opt(0) {
        Some(arg) => Some(
            arg.downcast::<JsBoolean, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .value(&mut cx),
        ),
        None => None,
    };

    let id = account_wrapper.account_id.clone();
    let (sender, receiver) = channel();

    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;
        let addresses = match unspent {
            Some(unspent) => {
                if unspent {
                    account.list_unspent_addresses().await.unwrap()
                } else {
                    account.list_spent_addresses().await.unwrap()
                }
            }
            None => account.addresses().await,
        };

        let mut result = vec![];
        for address in addresses.iter() {
            result.push(serde_json::to_string(address).unwrap());
        }
        let _ = sender.send(result);
    });

    let addresses = receiver.recv().unwrap();
    let js_array = JsArray::new(&mut cx, addresses.len() as u32);
    for (index, address) in addresses.iter().enumerate() {
        let addr = cx.string(address);
        js_array.set(&mut cx, index as u32, addr)?;
    }

    Ok(js_array)
}

pub fn set_alias(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let id = account_wrapper.account_id.clone();
    let alias = cx.argument::<JsString>(0)?.value(&mut cx);

    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        account_handle
            .set_alias(alias)
            .await
            .expect("failed to update account alias");
    });

    Ok(cx.undefined())
}

pub fn set_client_options(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );

    let client_options = cx.argument::<JsString>(0)?.value(&mut cx);
    let client_options = serde_json::from_str::<ClientOptionsDto>(&client_options).unwrap();

    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(id.as_str()).await;
        let result = account_handle
            .set_client_options(client_options.into())
            .await
            .expect("failed to update client options");
        let _ = sender.send(result);
    });
    let _ = receiver.recv().unwrap();

    Ok(cx.undefined())
}

pub fn get_message(mut cx: FunctionContext) -> JsResult<JsValue> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let message_id =
        MessageId::from_str(cx.argument::<JsString>(0)?.value(&mut cx).as_str()).expect("invalid message id");
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let account = account_handle.read().await;
        let message = account.get_message(&message_id).await;
        let _ = sender.send(message);
    });
    let message = receiver.recv().unwrap();

    match message {
        Some(m) => Ok(cx.string(serde_json::to_string(&m).unwrap()).as_value(&mut cx)),
        None => Ok(cx.undefined().as_value(&mut cx)),
    }
}

pub fn get_address(mut cx: FunctionContext) -> JsResult<JsValue> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let address = parse_address(cx.argument::<JsString>(0)?.value(&mut cx)).expect("invalid address");
    let id = account_wrapper.account_id.clone();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let account = account_handle.read().await;
        let address = account.addresses().iter().find(|a| a.address() == &address);

        let address = address.map(|a| serde_json::to_string(&a).unwrap());
        let _ = sender.send(address);
    });
    let address = receiver.recv().unwrap();

    match address {
        Some(a) => Ok(cx.string(a).as_value(&mut cx)),
        None => Ok(cx.undefined().as_value(&mut cx)),
    }
}

pub fn generate_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(account_wrapper.account_id.as_str()).await;
        let address = account_handle
            .generate_address()
            .await
            .expect("error generating address");
        let _ = sender.send(address);
    });
    let address = receiver.recv().unwrap();

    Ok(cx.string(serde_json::to_string(&address).unwrap()))
}

pub fn generate_addresses(mut cx: FunctionContext) -> JsResult<JsArray> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let id = account_wrapper.account_id.clone();
    let amount = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let addresses = account_handle
            .generate_addresses(amount)
            .await
            .expect("error generating address");
        let _ = sender.send(addresses);
    });

    let addresses = receiver.recv().unwrap();
    let js_array = JsArray::new(&mut cx, addresses.len() as u32);
    for (index, address) in addresses.iter().enumerate() {
        let msg = cx.string(serde_json::to_string(&address).unwrap());
        js_array.set(&mut cx, index as u32, msg)?;
    }

    Ok(js_array)
}

pub fn latest_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(account_wrapper.account_id.as_str()).await;
        let account = account_handle.read().await;
        let address = account.latest_address();
        let _ = sender.send(address.clone());
    });
    let address = receiver.recv().unwrap();

    Ok(cx.string(serde_json::to_string(&address).unwrap()))
}

pub fn get_unused_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let id = account_wrapper.account_id.clone();
    let (sender, receiver) = channel();

    crate::RUNTIME.spawn(async move {
        let account_handle = crate::get_account(&id).await;
        let address = account_handle.get_unused_address().await.unwrap();
        let _ = sender.send(serde_json::to_string(&address).unwrap());
    });

    Ok(cx.string(receiver.recv().unwrap()))
}

pub fn sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let (options, callback) = match cx.argument_opt(1) {
        Some(arg) => {
            let cb = arg
                .downcast::<JsFunction, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .root(&mut cx);
            let options = cx.argument::<JsString>(0)?;
            let options = serde_json::from_str::<SyncOptions>(options.value(&mut cx).as_str()).unwrap();
            (options, cb)
        }
        None => (Default::default(), cx.argument::<JsFunction>(0)?.root(&mut cx)),
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
        let _synced_account = synchronizer.execute().await.unwrap();

        account_wrapper.channel.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![cx.undefined().upcast::<JsValue>(), cx.string(id).upcast::<JsValue>()];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn send(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let address = cx.argument::<JsString>(0)?.value(&mut cx);
    let amount = cx.argument::<JsNumber>(1)?.value(&mut cx) as u64;
    let (options, cb) = match cx.argument_opt(3) {
        Some(arg) => {
            let cb = arg
                .downcast::<JsFunction, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .root(&mut cx);
            let options = cx.argument::<JsString>(2)?;
            let options = serde_json::from_str(&options.value(&mut cx)).unwrap();
            (options, cb)
        }
        None => (TransferOptions::default(), cx.argument::<JsFunction>(2)?.root(&mut cx)),
    };

    let mut transfer_builder = Transfer::builder(
        parse_address(address).expect("invalid address format"),
        NonZeroU64::new(amount).expect("amount can't be zero"),
        options.output_kind,
    )
    .with_remainder_value_strategy(options.remainder_value_strategy);

    if let Some(indexation) = options.indexation {
        transfer_builder = transfer_builder.with_indexation(
            IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default())
                .expect("index can't be empty"),
        );
    }

    if options.skip_sync {
        transfer_builder = transfer_builder.with_skip_sync();
    }

    let transfer = transfer_builder.finish();

    let id = account_wrapper.account_id.clone();
    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;

        let result = account.transfer(transfer.clone()).await;

        account_wrapper.channel.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();

            let args = match result {
                Ok(message) => {
                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.string(serde_json::to_string(&message).unwrap()).as_value(&mut cx),
                    ]
                }
                Err(e) => {
                    vec![
                        cx.string(e.to_string()).as_value(&mut cx),
                        cx.undefined().upcast::<JsValue>(),
                    ]
                }
            };

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn send_to_many(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let js_arr_handle: Handle<JsArray> = cx.argument(0)?;
    let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;
    let mut outputs = Vec::new();

    for js_value in vec {
        let js_object = js_value.downcast::<JsObject, FunctionContext>(&mut cx).unwrap();
        let address = js_object
            .get(&mut cx, "address")?
            .downcast::<JsString, FunctionContext>(&mut cx)
            .or_throw(&mut cx)?;
        let amount = js_object
            .get(&mut cx, "amount")?
            .downcast::<JsNumber, FunctionContext>(&mut cx)
            .or_throw(&mut cx)?;
        let output_kind = match js_object
            .get(&mut cx, "outputKind")?
            .downcast::<JsString, FunctionContext>(&mut cx)
        {
            Ok(value) => OutputKind::from_str(&value.value(&mut cx)).ok(),
            _ => None,
        };
        outputs.push(TransferOutput::new(
            parse_address(address.value(&mut cx)).expect("invalid address format"),
            NonZeroU64::new(amount.value(&mut cx) as u64).expect("amount can't be zero"),
            output_kind,
        ));
    }

    let (options, cb) = match cx.argument_opt(2) {
        Some(arg) => {
            let cb = arg
                .downcast::<JsFunction, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .root(&mut cx);
            let options = cx.argument::<JsString>(1)?.value(&mut cx);
            let options = serde_json::from_str(&options).unwrap();
            (options, cb)
        }
        None => (TransferOptions::default(), cx.argument::<JsFunction>(1)?.root(&mut cx)),
    };

    let mut transfer_builder = Transfer::builder_with_outputs(outputs)
        .expect("Outputs must be less then 125")
        .with_remainder_value_strategy(options.remainder_value_strategy);
    if let Some(indexation) = options.indexation {
        transfer_builder = transfer_builder.with_indexation(
            IndexationPayload::new(&indexation.index, &indexation.data.unwrap_or_default())
                .expect("index can't be empty"),
        );
    }
    if options.skip_sync {
        transfer_builder = transfer_builder.with_skip_sync();
    }

    let transfer = transfer_builder.finish();
    let id = account_wrapper.account_id.clone();

    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;

        let result = account.transfer(transfer.clone()).await;

        account_wrapper.channel.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();

            let args = match result {
                Ok(message) => {
                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.string(serde_json::to_string(&message).unwrap()).as_value(&mut cx),
                    ]
                }
                Err(e) => {
                    vec![
                        cx.string(e.to_string()).as_value(&mut cx),
                        cx.undefined().upcast::<JsValue>(),
                    ]
                }
            };

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

#[derive(Deserialize)]
pub enum RepostAction {
    Retry,
    Reattach,
    Promote,
}

pub fn repost(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );

    let message_id =
        MessageId::from_str(cx.argument::<JsString>(0)?.value(&mut cx).as_str()).expect("invalid message id");
    let action_type =
        serde_json::from_str::<RepostAction>(cx.argument::<JsString>(0)?.value(&mut cx).as_str()).unwrap();
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let id = account_wrapper.account_id.clone();
    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;
        let message = match action_type {
            RepostAction::Retry => account.retry(&message_id).await.unwrap(),
            RepostAction::Reattach => account.reattach(&message_id).await.unwrap(),
            RepostAction::Promote => account.promote(&message_id).await.unwrap(),
        };

        account_wrapper.channel.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                cx.undefined().upcast::<JsValue>(),
                cx.string(serde_json::to_string(&message).unwrap()).as_value(&mut cx),
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn consolidate_outputs(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let include_dust_allowance_output = cx.argument::<JsBoolean>(0)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let id = account_wrapper.account_id.clone();
    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;

        let result = account.consolidate_outputs(include_dust_allowance_output).await;

        account_wrapper.channel.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();

            let args = match result {
                Ok(messages) => {
                    let js_array = JsArray::new(&mut cx, messages.len() as u32);
                    for (index, message) in messages.iter().enumerate() {
                        let msg = cx.string(serde_json::to_string(message).unwrap());
                        js_array.set(&mut cx, index as u32, msg)?;
                    }

                    vec![cx.undefined().upcast::<JsValue>(), js_array.as_value(&mut cx)]
                }
                Err(e) => {
                    vec![
                        cx.string(e.to_string()).as_value(&mut cx),
                        cx.undefined().upcast::<JsValue>(),
                    ]
                }
            };

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn is_latest_address_unused(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let account_wrapper = Arc::clone(
        &&cx.this()
            .downcast_or_throw::<JsBox<Arc<AccountWrapper>>, FunctionContext>(&mut cx)?,
    );
    let cb = cx.argument::<JsFunction>(0)?.root(&mut cx);

    let id = account_wrapper.account_id.clone();
    crate::RUNTIME.spawn(async move {
        let account = crate::get_account(id.as_str()).await;

        let result = account.is_latest_address_unused().await;

        account_wrapper.channel.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();

            let args = match result {
                Ok(is_used) => {
                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.boolean(is_used).as_value(&mut cx),
                    ]
                }
                Err(e) => {
                    vec![
                        cx.string(e.to_string()).as_value(&mut cx),
                        cx.undefined().upcast::<JsValue>(),
                    ]
                }
            };

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

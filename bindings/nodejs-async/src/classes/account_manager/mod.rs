// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::ClientOptionsDto;
use std::{num::NonZeroU64, path::PathBuf, sync::Arc, time::Duration};

// use iota_client::Address;
use bee_message::address::Address;
use iota_wallet::{
    account::AccountIdentifier,
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    address::parse as parse_address,
    iota_migration::client::migration::{add_tryte_checksum, encode_migration_address},
    signing::SignerType,
    DateTime, Local,
};
use neon::{prelude::*, result::Throw};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::sync::mpsc::channel;
use tokio::sync::RwLock;

use super::AccountWrapper;

mod create_migration_bundle;
mod get_migration_data;
mod internal_transfer;
mod is_latest_address_unused;
mod send_migration_bundle;
mod sync;

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum AccountSignerType {
    Stronghold = 1,
}

impl Default for AccountSignerType {
    fn default() -> Self {
        Self::Stronghold
    }
}

#[derive(Deserialize)]
pub struct AccountToCreate {
    #[serde(rename = "clientOptions")]
    pub client_options: ClientOptionsDto,
    pub alias: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Local>>,
    #[serde(rename = "signerType", default)]
    pub signer_type: AccountSignerType,
    #[serde(rename = "skipPersistence", default)]
    pub skip_persistence: bool,
}

fn js_value_to_account_id(cx: &mut FunctionContext, value: Handle<JsValue>) -> NeonResult<AccountIdentifier> {
    match value.downcast::<JsString, FunctionContext>(cx) {
        Ok(js_string) => {
            let id = js_string.value(cx);
            Ok(id.into())
        }
        Err(_) => {
            let index: JsNumber = *value.downcast_or_throw(cx)?;
            Ok((index.value(cx) as usize).into())
        }
    }
}

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_FOLDER.into()
}

#[derive(Default, Deserialize)]
struct ManagerOptions {
    #[serde(rename = "storagePath", default = "default_storage_path")]
    storage_path: PathBuf,
    #[serde(rename = "storagePassword")]
    storage_password: Option<String>,
    #[serde(rename = "outputConsolidationThreshold")]
    output_consolidation_threshold: Option<usize>,
    #[serde(
        rename = "automaticOutputConsolidation",
        default = "default_automatic_output_consolidation"
    )]
    automatic_output_consolidation: bool,
    #[serde(rename = "syncSpentOutputs", default)]
    sync_spent_outputs: bool,
    #[serde(rename = "persistEvents", default)]
    persist_events: bool,
    #[serde(rename = "allowCreateMultipleEmptyAccounts", default)]
    allow_create_multiple_empty_accounts: bool,

    #[serde(rename = "skipPolling", default = "default_skip_polling")]
    skip_polling: bool,
    #[serde(rename = "pollingInterval")]
    polling_interval: Option<u64>,
}

fn default_automatic_output_consolidation() -> bool {
    true
}

fn default_skip_polling() -> bool {
    false
}

macro_rules! event_getter {
    ($cx: ident, $get_fn_name: ident) => {{
        let count = match $cx.argument_opt(0) {
            Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as usize,
            None => 0,
        };
        let skip = match $cx.argument_opt(1) {
            Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as usize,
            None => 0,
        };
        let from_timestamp = match $cx.argument_opt(2) {
            Some(arg) => Some(arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as i64),
            None => None,
        };

        let events = {
            let this = $cx.this();
            let guard = $cx.lock();
            let ref_ = &this.borrow(&guard).0;
            crate::block_on(async move {
                let manager = ref_.read().await;
                manager.$get_fn_name(count, skip, from_timestamp).await.unwrap()
            })
        };

        let js_array = JsArray::new(&mut $cx, events.len() as u32);
        for (index, event) in events.into_iter().enumerate() {
            let js_event = neon_serde::to_value(&mut $cx, &event)?;
            js_array.set(&mut $cx, index as u32, js_event)?;
        }

        Ok(js_array.upcast())
    }};
}

macro_rules! event_count_getter {
    ($cx: ident, $get_fn_name: ident) => {{
        let from_timestamp = match $cx.argument_opt(0) {
            Some(arg) => Some(arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as i64),
            None => None,
        };

        let count = {
            let this = $cx.this();
            let guard = $cx.lock();
            let ref_ = &this.borrow(&guard).0;
            crate::block_on(async move {
                let manager = ref_.read().await;
                manager.$get_fn_name(from_timestamp).await.unwrap()
            })
        };

        Ok($cx.number(count as f64).upcast())
    }};
}

pub struct AccountManagerWrapper {
    queue: EventQueue,
    account_manager: AccountManager,
}

impl Finalize for AccountManagerWrapper {}

impl AccountManagerWrapper {
    fn new(queue: EventQueue, options: String) -> Arc<Self> {
        log::debug!("------------------------------------- AccountManagerWrapper");
        let options = match serde_json::from_str::<crate::types::ManagerOptions>(&options) {
            Ok(options) => options,
            Err(e) => {
                log::debug!(
                    "------------------------------------- AccountManagerWrapper error - {:?}",
                    e
                );
                crate::types::ManagerOptions::default()
            }
        };

        let mut manager = AccountManager::builder()
            .with_storage(&options.storage_path, options.storage_password.as_deref())
            .expect("failed to init storage");
        if !options.automatic_output_consolidation {
            manager = manager.with_automatic_output_consolidation_disabled();
        }
        if options.sync_spent_outputs {
            manager = manager.with_sync_spent_outputs();
        }
        if options.persist_events {
            manager = manager.with_event_persistence();
        }
        if options.allow_create_multiple_empty_accounts {
            manager = manager.with_multiple_empty_accounts();
        }
        if options.skip_polling {
            manager = manager.with_skip_polling();
        }
        if let Some(polling_interval) = options.polling_interval {
            manager = manager.with_polling_interval(Duration::from_secs(polling_interval));
        }
        if let Some(threshold) = options.output_consolidation_threshold {
            manager = manager.with_output_consolidation_threshold(threshold);
        }
        let manager = crate::RUNTIME
            .block_on(manager.finish())
            .expect("error initializing account manager");

        log::debug!("------------------------------------- AccountManagerWrapper end");
        Arc::new(Self {
            queue,
            account_manager: manager,
        })
    }
}

pub fn account_manager_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<AccountManagerWrapper>>> {
    log::debug!("------------------------------------- account_manager_new");
    let options = cx.argument::<JsString>(0)?;
    let options = options.value(&mut cx);
    let queue = cx.queue();
    let account_wrapper = AccountManagerWrapper::new(queue, options);

    Ok(cx.boxed(account_wrapper))
}

pub fn start_background_sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let polling_interval = cx.argument::<JsNumber>(0)?.value(&mut cx) as u64;
    let automatic_output_consolidation = cx.argument::<JsBoolean>(1)?.value(&mut cx);

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let result = wrapper
            .account_manager
            .start_background_sync(Duration::from_secs(polling_interval), automatic_output_consolidation)
            .await;
        let _ = sender.send(result);
    });

    let _ = receiver.recv().unwrap();
    Ok(cx.undefined())
}

pub fn stop_background_sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);

    wrapper.account_manager.stop_background_sync();

    Ok(cx.undefined())
}

pub fn set_storage_password(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let password = cx.argument::<JsString>(0)?.value(&mut cx);

    crate::RUNTIME.spawn(async move { wrapper.account_manager.set_storage_password(password).await });

    Ok(cx.undefined())
}

pub fn set_stronghold_password(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let password = cx.argument::<JsString>(0)?.value(&mut cx);

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let result = wrapper.account_manager.set_stronghold_password(password).await;
        let _ = sender.send(result);
    });
    let _ = receiver.recv().unwrap();
    Ok(cx.undefined())
}

pub fn change_stronghold_password(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let current_password = cx.argument::<JsString>(0)?.value(&mut cx);
    let new_password = cx.argument::<JsString>(1)?.value(&mut cx);
    let (sender, receiver) = channel();

    crate::RUNTIME.spawn(async move {
        let result = wrapper
            .account_manager
            .change_stronghold_password(current_password, new_password)
            .await;
        let _ = sender.send(result);
    });
    let _ = receiver.recv().unwrap();
    Ok(cx.undefined())
}

pub fn generate_mnemonic(mut cx: FunctionContext) -> JsResult<JsString> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let mnemonic = wrapper
        .account_manager
        .generate_mnemonic()
        .expect("failed to generate mnemonic");

    Ok(cx.string(&mnemonic))
}

pub fn store_mnemonic(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let signer_type = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    let signer_type: AccountSignerType = serde_json::from_str(&signer_type.to_string()).expect("invalid signer type");
    let signer_type = match signer_type {
        AccountSignerType::Stronghold => SignerType::Stronghold,
    };

    let mnemonic = match cx.argument_opt(1) {
        Some(_) => Some(cx.argument::<JsString>(1)?.value(&mut cx)),
        None => None,
    };

    crate::RUNTIME.spawn(async move {
        let _ = wrapper.account_manager.store_mnemonic(signer_type, mnemonic).await;
    });

    Ok(cx.undefined())
}

pub fn create_account(mut cx: FunctionContext) -> JsResult<JsBox<Arc<crate::account::AccountWrapper>>> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let account_to_create = cx.argument::<JsString>(0)?;
    let account_to_create = account_to_create.value(&mut cx);
    let account_to_create = serde_json::from_str::<AccountToCreate>(&account_to_create).unwrap();

    // log::debug!(&account_to_create);

    let mut builder = wrapper
        .account_manager
        .create_account(account_to_create.client_options.into())
        .expect("failed to create account")
        .signer_type(match account_to_create.signer_type {
            AccountSignerType::Stronghold => SignerType::Stronghold,
        });
    if let Some(alias) = &account_to_create.alias {
        builder = builder.alias(alias);
    }
    if let Some(created_at) = &account_to_create.created_at {
        builder = builder.created_at(*created_at);
    }
    if account_to_create.skip_persistence {
        builder = builder.skip_persistence();
    }

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account = builder.initialise().await;
        let result = match account {
            Ok(account) => {
                let id = crate::store_account(account).await;
                Ok(id)
            }
            Err(e) => Err(e.to_string()),
        };

        let _ = sender.send(result);
    });
    let result = receiver.recv().unwrap();

    match result {
        Ok(id) => {
            let queue = cx.queue();
            let account_wrapper = crate::account::AccountWrapper::new(queue, id);
            Ok(cx.boxed(account_wrapper))
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn get_account(mut cx: FunctionContext) -> JsResult<JsBox<Arc<crate::account::AccountWrapper>>> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let id = cx.argument::<JsValue>(0)?;
    let id = js_value_to_account_id(&mut cx, id)?;

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let account = wrapper.account_manager.get_account(id).await;
        let result = match account {
            Ok(account) => {
                let id = crate::store_account(account).await;
                Ok(id)
            }
            Err(e) => Err(e.to_string()),
        };

        let _ = sender.send(result);
    });

    let result = receiver.recv().unwrap();

    match result {
        Ok(id) => {
            let queue = cx.queue();
            let account_wrapper = crate::account::AccountWrapper::new(queue, id);
            Ok(cx.boxed(account_wrapper))
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn get_accounts(mut cx: FunctionContext) -> JsResult<JsArray> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let accounts = wrapper.account_manager.get_accounts().await.unwrap();
        let mut ids = vec![];
        for account in accounts.into_iter() {
            ids.push(crate::store_account(account).await);
        }
        let _ = sender.send(ids);
    });
    let ids = receiver.recv().unwrap();

    let js_array = JsArray::new(&mut cx, ids.len() as u32);
    for (index, id) in ids.into_iter().enumerate() {
        let queue = cx.queue();
        let account_wrapper = crate::account::AccountWrapper::new(queue, id);
        let boxed = cx.boxed(account_wrapper);
        js_array.set(&mut cx, index as u32, boxed)?;
    }

    Ok(js_array)
}

pub fn remove_account(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let id = cx.argument::<JsValue>(0)?;
    let id = js_value_to_account_id(&mut cx, id)?;
    let (sender, receiver) = channel();
        crate::RUNTIME.spawn(async move {
            let result = wrapper.account_manager.remove_account(id).await;
            let _ = sender.send(result);
        });

    let _ = receiver.recv().unwrap();
    Ok(cx.undefined())
}

pub fn sync_accounts(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let (options, cb) = match cx.argument_opt(1) {
        Some(arg) => {
            let cb = arg.downcast::<JsFunction, FunctionContext>(&mut cx).or_throw(&mut cx)?;
            let options = cx.argument::<JsString>(0)?.value(&mut cx);
            let options = serde_json::from_str::<crate::account::SyncOptions>(&options).unwrap();
            (options, cb.root(&mut cx))
        }
        None => (Default::default(), cx.argument::<JsFunction>(0)?.root(&mut cx)),
    };

    crate::RUNTIME.spawn(async move {
        let mut synchronizer = wrapper.account_manager.sync_accounts().unwrap();
        if let Some(address_index) = options.address_index {
            synchronizer = synchronizer.address_index(address_index);
        }
        if let Some(gap_limit) = options.gap_limit {
            synchronizer = synchronizer.gap_limit(gap_limit);
        }

        let result = match synchronizer.execute().await {
            Ok(synced_accounts) => {
                let mut ids = vec![];
                for synced_account in synced_accounts {
                    let id = crate::store_synced_account(synced_account).await;
                    ids.push(id);
                }

                Ok(ids)
            },
            Err(e) => Err(e.to_string()),
        };

        wrapper.queue.send(move |mut cx| {

            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(ids) => {

                    let js_array = JsArray::new(&mut cx, ids.len() as u32);
                    for (index, id) in ids.iter().enumerate() {
                        let id = cx.string(id);
                        js_array.set(&mut cx, index as u32, id).unwrap();
                    }

                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        js_array.as_value(&mut cx)
                    ]
                },
                Err(e) => {
                    vec![
                        cx.string(e).as_value(&mut cx),
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

pub fn internal_transfer(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let from_account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(0)?);
    let to_account_wrapper = Arc::clone(&&cx.argument::<JsBox<Arc<AccountWrapper>>>(1)?);
    let amount = cx.argument::<JsNumber>(2)?.value(&mut cx) as u64;
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let from_account_id = from_account_wrapper.account_id.clone();
    let to_account_id = to_account_wrapper.account_id.clone();

    crate::RUNTIME.spawn(async move {
        let result = wrapper.account_manager
                .internal_transfer(&from_account_id, &to_account_id, NonZeroU64::new(amount).unwrap())
                .await;

        wrapper.queue.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(message) => {
                    let msg = serde_json::to_string(&message).unwrap();
                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.string(msg).as_value(&mut cx),
                    ]
                },
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

pub fn backup(mut cx: FunctionContext) -> JsResult<JsString> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let backup_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let password = cx.argument::<JsString>(1)?.value(&mut cx);

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let result = wrapper.account_manager.backup(backup_path, password).await;
        let result = match result {
            Ok(path) => Ok(path.display().to_string()),
            Err(e) => Err(e.to_string()),
        };

        let _ = sender.send(result);
    });

    match receiver.recv().unwrap() {
        Ok(path) => Ok(cx.string(path)),
        Err(e) => cx.throw_error(e),
    }
}

pub fn import_accounts(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let source = cx.argument::<JsString>(0)?.value(&mut cx);
    let password = cx.argument::<JsString>(1)?.value(&mut cx);

    crate::RUNTIME.spawn(async move {
        let _ = wrapper.account_manager.import_accounts(source, password).await;
    });

    Ok(cx.undefined())
}

pub fn is_latest_address_unused(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let cb = cx.argument::<JsFunction>(0)?.root(&mut cx);

    crate::RUNTIME.spawn(async move {
        let result = wrapper.account_manager.is_latest_address_unused().await;

        wrapper.queue.send(move |mut cx| {
            let cb = cb.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(is_unused) => {
                    vec![
                        cx.undefined().upcast::<JsValue>(),
                        cx.boolean(is_unused).as_value(&mut cx),
                    ]
                },
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



pub fn set_client_options(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let wrapper = Arc::clone(&&cx.this().downcast_or_throw::<JsBox<Arc<AccountManagerWrapper>>, FunctionContext>(&mut cx)?);
    let client_options = cx.argument::<JsString>(0)?.value(&mut cx);
    let client_options = serde_json::from_str::<ClientOptionsDto>(&client_options).unwrap();

    let (sender, receiver) = channel();
    crate::RUNTIME.spawn(async move {
        let result = wrapper
            .account_manager
            .set_client_options(client_options.into()).await;
        let _ = sender.send(result);
    });
    let _ = receiver.recv().unwrap();
    Ok(cx.undefined())
}

// fn getBalanceChangeEvents(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_getter!(cx, get_balance_change_events)
// }

// fn getBalanceChangeEventCount(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_count_getter!(cx, get_balance_change_event_count)
// }

// fn getTransactionConfirmationEvents(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_getter!(cx, get_transaction_confirmation_events)
// }

// fn getTransactionConfirmationEventCount(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_count_getter!(cx, get_transaction_confirmation_event_count)
// }

// fn getNewTransactionEvents(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_getter!(cx, get_new_transaction_events)
// }

// fn getNewTransactionEventCount(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_count_getter!(cx, get_new_transaction_event_count)
// }

// fn getReattachmentEvents(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_getter!(cx, get_reattachment_events)
// }

// fn getReattachmentEventCount(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_count_getter!(cx, get_reattachment_event_count)
// }

// fn getBroadcastEvents(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_getter!(cx, get_broadcast_events)
// }

// fn getBroadcastEventCount(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     event_count_getter!(cx, get_broadcast_event_count)
// }
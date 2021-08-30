// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::needless_borrow)]

mod classes;
use classes::*;
pub mod types;

use neon::prelude::*;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::{runtime::Runtime, sync::RwLock};

pub use iota_wallet::{
    account::{AccountHandle, SyncedAccount},
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    actor::{AccountIdentifier, Message as WalletMessage, MessageType, Response, ResponseType, WalletMessageHandler},
    address::parse as parse_address,
    event::{
        on_balance_change, on_broadcast, on_confirmation_state_change, on_error, on_migration_progress,
        on_new_transaction, on_reattachment, on_stronghold_status_change, on_transfer_progress,
        remove_balance_change_listener, remove_broadcast_listener, remove_confirmation_state_change_listener,
        remove_error_listener, remove_migration_progress_listener, remove_new_transaction_listener,
        remove_reattachment_listener, remove_stronghold_status_change_listener, remove_transfer_progress_listener,
        EventId,
    },
    message::{IndexationPayload, MessageId, RemainderValueStrategy, Transfer, TransferOutput},
    Error,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

type AccountInstanceMap = Arc<RwLock<HashMap<String, AccountHandle>>>;
type SyncedAccountHandle = Arc<RwLock<SyncedAccount>>;
type SyncedAccountInstanceMap = Arc<RwLock<HashMap<String, SyncedAccountHandle>>>;

/// Gets the account instances map.
fn account_instances() -> &'static AccountInstanceMap {
    static INSTANCES: Lazy<AccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) async fn get_account(id: &str) -> AccountHandle {
    account_instances()
        .read()
        .await
        .get(id)
        .expect("account dropped or not initialised")
        .clone()
}

pub(crate) async fn store_account(account_handle: AccountHandle) -> String {
    let handle = account_handle.clone();
    let id = handle.id().await;

    account_instances().write().await.insert(id.clone(), account_handle);

    id
}

/// Gets the synced account instances map.
fn synced_account_instances() -> &'static SyncedAccountInstanceMap {
    static INSTANCES: Lazy<SyncedAccountInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

#[allow(dead_code)]
pub(crate) async fn get_synced_account(id: &str) -> SyncedAccountHandle {
    synced_account_instances()
        .read()
        .await
        .get(id)
        .expect("synced account dropped or not initialised")
        .clone()
}

pub(crate) async fn store_synced_account(synced_account: SyncedAccount) -> String {
    let mut map = synced_account_instances().write().await;
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(10)
        .collect();
    map.insert(id.clone(), Arc::new(RwLock::new(synced_account)));
    id
}

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

use bee_common::logger::{logger_init, LoggerConfigBuilder};

pub fn init_logger(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsString>(0)?.value(&mut cx);
    let config: LoggerConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    logger_init(config.finish()).expect("failed to init logger");
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Account methods.
    cx.export_function("accountNew", classes::account::account_new)?;
    cx.export_function("id", classes::account::id)?;
    cx.export_function("index", classes::account::index)?;
    cx.export_function("alias", classes::account::alias)?;
    cx.export_function("messageCount", classes::account::message_count)?;
    cx.export_function("sync", classes::account::sync)?;
    cx.export_function("generateAddress", classes::account::generate_address)?;
    cx.export_function("latestAddress", classes::account::latest_address)?;
    cx.export_function("getNodeInfo", classes::account::get_node_info)?;
    cx.export_function("listMessages", classes::account::list_messages)?;
    cx.export_function("listAddresses", classes::account::list_addresses)?;
    cx.export_function("setAlias", classes::account::set_alias)?;
    cx.export_function("setClientOptions", classes::account::set_client_options)?;
    cx.export_function("getMessage", classes::account::get_message)?;
    cx.export_function("getAddress", classes::account::get_address)?;
    cx.export_function("getUnusedAddress", classes::account::get_unused_address)?;
    cx.export_function("generateAddresses", classes::account::generate_addresses)?;
    cx.export_function("send", classes::account::send)?;
    cx.export_function("sendToMany", classes::account::send_to_many)?;
    cx.export_function("isLatestAddressUnused", classes::account::is_latest_address_unused)?;
    cx.export_function("consolidateOutputs", classes::account::consolidate_outputs)?;
    cx.export_function("repost", classes::account::repost)?;
    cx.export_function("balance", classes::account::balance)?;

    // Account manager methods.
    cx.export_function("accountManagerNew", classes::account_manager::account_manager_new)?;
    cx.export_function("getAccount", classes::account_manager::get_account)?;
    cx.export_function("getAccounts", classes::account_manager::get_accounts)?;
    cx.export_function("removeAccount", classes::account_manager::remove_account)?;
    cx.export_function("syncAccounts", classes::account_manager::sync_accounts)?;
    cx.export_function("createAccount", classes::account_manager::create_account)?;
    cx.export_function("internalTransfer", classes::account_manager::internal_transfer)?;
    cx.export_function("setClientOptionsManager", classes::account_manager::set_client_options)?;
    cx.export_function(
        "isLatestAddressUnused",
        classes::account_manager::is_latest_address_unused,
    )?;
    cx.export_function(
        "setStrongholdPassword",
        classes::account_manager::set_stronghold_password,
    )?;
    cx.export_function("storeMnemonic", classes::account_manager::store_mnemonic)?;
    cx.export_function("backup", classes::account_manager::backup)?;
    cx.export_function("importAccounts", classes::account_manager::import_accounts)?;
    cx.export_function("setStoragePassword", classes::account_manager::set_storage_password)?;
    cx.export_function(
        "changeStrongholdPassword",
        classes::account_manager::change_stronghold_password,
    )?;
    cx.export_function("generateMnemonic", classes::account_manager::generate_mnemonic)?;
    cx.export_function("startBackgroundSync", classes::account_manager::start_background_sync)?;
    cx.export_function("stopBackgroundSync", classes::account_manager::stop_background_sync)?;

    cx.export_function(
        "getBalanceChangeEvents",
        classes::account_manager::get_balance_change_events,
    )?;
    cx.export_function(
        "getBalanceChangeEventCount",
        classes::account_manager::get_balance_change_event_count,
    )?;
    cx.export_function(
        "getTransactionConfirmationEvents",
        classes::account_manager::get_transaction_confirmation_events,
    )?;
    cx.export_function(
        "getTransactionConfirmationEventCount",
        classes::account_manager::get_transaction_confirmation_event_count,
    )?;
    cx.export_function(
        "getNewTransactionEvents",
        classes::account_manager::get_new_transaction_events,
    )?;
    cx.export_function(
        "getNewTransactionEventCount",
        classes::account_manager::get_new_transaction_event_count,
    )?;
    cx.export_function(
        "getReattachmentEvents",
        classes::account_manager::get_reattachment_events,
    )?;
    cx.export_function(
        "getReattachmentEventCount",
        classes::account_manager::get_reattachment_event_count,
    )?;
    cx.export_function("getBroadcastEvents", classes::account_manager::get_broadcast_events)?;
    cx.export_function(
        "getBroadcastEventCount",
        classes::account_manager::get_broadcast_event_count,
    )?;

    // Message handler methods.
    cx.export_function("sendMessage", classes::message_handler::send_message)?;
    cx.export_function("messageHandlerNew", classes::message_handler::message_handler_new)?;

    cx.export_function("eventListenerNew", classes::event_listener::event_listener_new)?;
    cx.export_function("listen", classes::event_listener::listen)?;
    cx.export_function("removeEventListeners", classes::event_listener::remove_event_listeners)?;

    cx.export_function("initLogger", init_logger)?;
    Ok(())
}

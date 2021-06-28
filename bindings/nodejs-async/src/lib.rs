/// The event module.
pub mod event_listeners;

use neon::prelude::*;
use std::sync::Arc;

use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

use serde::Deserialize;

use once_cell::sync::Lazy;
use std::{path::PathBuf, time::Duration};

pub use iota_wallet::{
    account_manager::{AccountManager, DEFAULT_STORAGE_FOLDER},
    actor::{Message as WalletMessage, MessageType, Response, ResponseType, WalletMessageHandler, AccountIdentifier},
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

use iota_client::common::logger::{logger_init, LoggerConfigBuilder};

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// #[derive(Deserialize)]
// pub struct ClientOptionsDto {
//     #[serde(rename = "primaryNode")]
//     primary_node: Option<NodeDto>,
//     #[serde(rename = "primaryPoWNode")]
//     primary_pow_node: Option<NodeDto>,
//     node: Option<NodeDto>,
//     #[serde(default)]
//     nodes: Vec<NodeDto>,
//     #[serde(rename = "nodePoolUrls", default)]
//     node_pool_urls: Vec<Url>,
//     network: Option<String>,
//     #[serde(rename = "mqttBrokerOptions")]
//     mqtt_broker_options: Option<BrokerOptions>,
//     #[serde(rename = "localPow", default = "default_local_pow")]
//     local_pow: bool,
//     #[serde(rename = "nodeSyncInterval")]
//     node_sync_interval: Option<Duration>,
//     #[serde(rename = "nodeSyncEnabled", default = "default_node_sync_enabled")]
//     node_sync_enabled: bool,
//     #[serde(rename = "requestTimeout")]
//     request_timeout: Option<Duration>,
//     #[serde(rename = "apiTimeout", default)]
//     api_timeout: HashMap<Api, Duration>,
// }

// #[derive(Deserialize)]
// pub struct AccountToCreate {
//     #[serde(rename = "clientOptions")]
//     pub client_options: ClientOptionsDto,
//     pub alias: Option<String>,
//     #[serde(rename = "createdAt")]
//     pub created_at: Option<DateTime<Local>>,
//     #[serde(rename = "signerType", default)]
//     pub signer_type: AccountSignerType,
//     #[serde(rename = "skipPersistence", default)]
//     pub skip_persistence: bool,
// }

// fn js_value_to_account_id(
//     cx: &mut CallContext<'_, JsAccountManager>,
//     value: Handle<JsValue>,
// ) -> NeonResult<AccountIdentifier> {
//     match value.downcast::<JsString>() {
//         Ok(js_string) => {
//             let id = js_string.value();
//             Ok(id.into())
//         }
//         Err(_) => {
//             let index: JsNumber = *value.downcast_or_throw(cx)?;
//             Ok((index.value() as usize).into())
//         }
//     }
// }

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

// macro_rules! event_getter {
//     ($cx: ident, $get_fn_name: ident) => {{
//         let count = match $cx.argument_opt(0) {
//             Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as usize,
//             None => 0,
//         };
//         let skip = match $cx.argument_opt(1) {
//             Some(arg) => arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as usize,
//             None => 0,
//         };
//         let from_timestamp = match $cx.argument_opt(2) {
//             Some(arg) => Some(arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as i64),
//             None => None,
//         };

//         let events = {
//             let this = $cx.this();
//             let guard = $cx.lock();
//             let ref_ = &this.borrow(&guard).0;
//             crate::block_on(async move {
//                 let manager = ref_.read().await;
//                 manager.$get_fn_name(count, skip, from_timestamp).await.unwrap()
//             })
//         };

//         let js_array = JsArray::new(&mut $cx, events.len() as u32);
//         for (index, event) in events.into_iter().enumerate() {
//             let js_event = neon_serde::to_value(&mut $cx, &event)?;
//             js_array.set(&mut $cx, index as u32, js_event)?;
//         }

//         Ok(js_array.upcast())
//     }};
// }

// macro_rules! event_count_getter {
//     ($cx: ident, $get_fn_name: ident) => {{
//         let from_timestamp = match $cx.argument_opt(0) {
//             Some(arg) => Some(arg.downcast::<JsNumber>().or_throw(&mut $cx)?.value() as i64),
//             None => None,
//         };

//         let count = {
//             let this = $cx.this();
//             let guard = $cx.lock();
//             let ref_ = &this.borrow(&guard).0;
//             crate::block_on(async move {
//                 let manager = ref_.read().await;
//                 manager.$get_fn_name(from_timestamp).await.unwrap()
//             })
//         };

//         Ok($cx.number(count as f64).upcast())
//     }};
// }

#[derive(Deserialize, Clone)]
pub(crate) struct DispatchMessage {
    pub(crate) id: String,
    #[serde(flatten)]
    pub(crate) message: MessageType,
}

struct AccountManagerWrapper {
    queue: EventQueue,
    message_handler: WalletMessageHandler,
}

impl Finalize for AccountManagerWrapper {}
impl AccountManagerWrapper {
    fn new(queue: EventQueue, options: String) -> Arc<Self> {
        let options = match serde_json::from_str::<ManagerOptions>(&options) {
            Ok(options) => options,
            Err(e) => {
                log::debug!("{:?}", e);
                ManagerOptions::default()
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
        let manager = RUNTIME
            .block_on(manager.finish())
            .expect("error initializing account manager");
        let message_handler = WalletMessageHandler::with_manager(manager);

        Arc::new(Self { queue, message_handler })
    }

    async fn send_message(&self, serialized_message: String) -> (String, bool) {
        log::debug!("{}", serialized_message);
        match serde_json::from_str::<DispatchMessage>(&serialized_message) {
            Ok(message) => {
                let (response_tx, mut response_rx) = unbounded_channel();
                log::debug!("--------------------------{:?}", &message.message);
                // https://damad.be/joost/blog/rust-serde-deserialization-of-an-enum-variant.html
                let wallet_message = WalletMessage::new(message.id.clone(), message.message.clone(), response_tx);

                log::debug!("--------------------------{:?}", &wallet_message);
                self.message_handler.handle(wallet_message).await;
                let response = response_rx.recv().await;
                if let Some(res) = response {
                    let mut is_err = match &res.response() {
                        ResponseType::Error(_) => true,
                        ResponseType::Panic(_) => true,
                        _ => false,
                    };

                    let msg = match serde_json::to_string(&res) {
                        Ok(msg) => msg,
                        Err(e) => {
                            is_err = true;
                            serde_json::to_string(&Response::new(
                                message.id,
                                message.message,
                                ResponseType::Error(e.into()),
                            ))
                            .expect("The response is generated manually, so unwrap is safe.")
                        }
                    };

                    (msg, is_err)
                } else {
                    ("No response".to_string(), true)
                }
            }
            Err(e) => {
                log::debug!("{:?}", e);
                (format!("Couldn't parse to message with error - {:?}", e), true)
            }
        }
    }
}

fn message_handler_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<AccountManagerWrapper>>> {
    let options = cx.argument::<JsString>(0)?;
    let options = options.value(&mut cx).to_string();
    let queue = cx.queue();
    let message_handler = AccountManagerWrapper::new(queue, options);

    Ok(cx.boxed(message_handler))
}

fn send_message(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let message = cx.argument::<JsString>(0)?;
    let message = message.value(&mut cx);
    let message_handler = Arc::clone(&&cx.argument::<JsBox<Arc<AccountManagerWrapper>>>(1)?);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    RUNTIME.spawn(async move {
        let (response, is_error) = message_handler.send_message(message).await;
        log::debug!("{:?}", response);
        message_handler.queue.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                if is_error {
                    cx.string(response.clone()).upcast::<JsValue>()
                } else {
                    cx.undefined().upcast::<JsValue>()
                },
                cx.string(response.clone()).upcast::<JsValue>(),
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn init_logger(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsString>(0)?.value(&mut cx);
    let config: LoggerConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    logger_init(config.finish()).expect("failed to init logger");
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("sendMessage", send_message)?;
    cx.export_function("messageHandlerNew", message_handler_new)?;
    cx.export_function("eventListenerNew", crate::event_listeners::event_listener_new)?;
    cx.export_function("initLogger", init_logger)?;
    cx.export_function("listen", crate::event_listeners::listen)?;
    cx.export_function("removeEventListeners", crate::event_listeners::remove_event_listeners)?;
    Ok(())
}

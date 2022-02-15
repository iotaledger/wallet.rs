use iota_wallet::actor::{Message, MessageType};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    sync::Mutex
};

mod wallet;

// use bee_common::logger::{LoggerConfig, LoggerOutputConfigBuilder};
// use log::LevelFilter;
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::mpsc::UnboundedSender};

use iota_wallet::account_manager::AccountManager;

type Callback = extern "C" fn(*const c_char, *mut c_void);
type IotaWalletTx = UnboundedSender<Message>;

#[derive(Debug)]
struct CallbackContext {
    data: *mut c_void,
}

unsafe impl Send for CallbackContext {}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

#[no_mangle]
pub extern "C" fn iota_initialize(storage_path: *const c_char) -> *mut IotaWalletTx {
    let c_storage_path: Option<&str> = if storage_path.is_null() {
        None
    } else {
        let c_storage_path = unsafe { CStr::from_ptr(storage_path) };
        Some(c_storage_path.to_str().unwrap())
    };

    let manager = block_on(async move {
        if let Some(path) = c_storage_path {
            AccountManager::builder().with_storage_folder(path).finish().await
        } else {
            AccountManager::builder().finish().await
        }
    });

    match manager {
        Ok(manager) => {
            let tx = wallet::spawn_actor(manager);
            Box::into_raw(Box::new(tx))
        }
        _ => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn iota_destroy(tx: *mut IotaWalletTx) {
    if tx.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(tx);
    }
}

#[no_mangle]
pub extern "C" fn iota_send_message(
    tx: *mut IotaWalletTx,
    message: *const c_char,
    callback: Callback,
    context: *mut c_void,
) {
    let (tx, c_message) = unsafe {
        if tx.is_null() || message.is_null() {
            return callback(std::ptr::null(), context);
        }

        let tx = if let Some(tx) = tx.as_ref() {
            tx
        } else {
            return callback(std::ptr::null(), context);
        };

        (tx, CStr::from_ptr(message))
    };

    let message = c_message.to_str().unwrap();
    let message_type = match serde_json::from_str::<MessageType>(message) {
        Ok(message_type) => message_type,
        Err(e) => {
            eprintln!("{:?}", e);
            return callback(std::ptr::null(), context);
        }
    };

    let callback_context = CallbackContext { data: context };

    wallet::runtime().spawn(async move {
        let callback_context = callback_context;
        let response = wallet::send_message(tx, message_type).await;
        let response = serde_json::to_string(&response).unwrap();
        let response = CString::new(response).unwrap();
        callback(response.as_ptr(), callback_context.data); // Callback from another thread, data better be useable from any thread
    });
}

// #[no_mangle]
// pub extern "C" fn iota_listen(actor_id: *const c_char, id: *const c_char, event_name: *const c_char) {
//     let c_actor_id = unsafe {
//         assert!(!actor_id.is_null());
//         CStr::from_ptr(actor_id)
//     };
//     let actor_id = c_actor_id.to_str().unwrap();

//     let c_id = unsafe {
//         assert!(!id.is_null());
//         CStr::from_ptr(id)
//     };
//     let id = c_id.to_str().unwrap();

//     let c_event_name = unsafe {
//         assert!(!event_name.is_null());
//         CStr::from_ptr(event_name)
//     };
//     let event_name = c_event_name.to_str().unwrap();

//     let event_type: EventType = event_name.try_into().expect("unknown event name");
//     // block_on(add_event_listener(actor_id, id, event_type));
// }

// #[no_mangle]
// pub extern "C" fn iota_init_logger(file_name: *const c_char) {
//     let c_file_name = unsafe {
//         assert!(!file_name.is_null());
//         CStr::from_ptr(file_name)
//     };
//     let _file_name = c_file_name.to_str().unwrap();

//     let output_config = LoggerOutputConfigBuilder::new()
//         .name(file_name)
//         .level_filter(LevelFilter::Debug);
//     init_logger(LoggerConfig::build().with_output(output_config));
// }

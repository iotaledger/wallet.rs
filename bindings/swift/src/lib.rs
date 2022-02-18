// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    message_interface,
    message_interface::{MessageType, WalletMessageHandler},
};

use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
};

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::runtime::Runtime;

// use bee_common::logger::{LoggerConfig, LoggerOutputConfigBuilder};
// use log::LevelFilter;

type Callback = extern "C" fn(message: *const c_char, error: *const c_char, context: *mut c_void);
type IotaWalletHandle = WalletMessageHandler;

#[derive(Debug)]
struct CallbackContext {
    data: *mut c_void,
}

unsafe impl Send for CallbackContext {}

fn runtime() -> &'static Arc<Runtime> {
    static INSTANCE: OnceCell<Arc<Runtime>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("org.iota.binding.thread")
                // Remove worker_threads restriction or bump it up depending on performance requirement
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
        )
    })
}

#[no_mangle]
pub extern "C" fn iota_initialize(storage_path: *const c_char) -> *mut IotaWalletHandle {
    let path = if storage_path.is_null() {
        None
    } else {
        let c_storage_path = unsafe { CStr::from_ptr(storage_path) };
        Some(c_storage_path.to_str().unwrap().to_string())
    };

    let handle = runtime().block_on(message_interface::create_message_handler(path));

    match handle {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        _ => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn iota_destroy(handle: *mut IotaWalletHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(handle);
    }
}
/// warning: callback will be invoked from another thread so context must be safe to `Send` across threads
#[no_mangle]
pub extern "C" fn iota_send_message(
    handle: *mut IotaWalletHandle,
    message: *const c_char,
    callback: Callback,
    context: *mut c_void,
) {
    let (handle, c_message) = unsafe {
        if handle.is_null() || message.is_null() {
            let error = CString::new("Null error; either wallet handle or message is invalid").unwrap();
            return callback(std::ptr::null(), error.as_ptr(), context);
        }

        let handle = if let Some(handle) = handle.as_ref() {
            handle
        } else {
            let error = CString::new("Invalid handle").unwrap();
            return callback(std::ptr::null(), error.as_ptr(), context);
        };

        (handle, CStr::from_ptr(message))
    };

    let message = c_message.to_str().unwrap();
    let message_type = match serde_json::from_str::<MessageType>(message) {
        Ok(message_type) => message_type,
        Err(e) => {
            let error = CString::new(format!("{:?}", e)).unwrap();
            return callback(std::ptr::null(), error.as_ptr(), context);
        }
    };

    let callback_context = CallbackContext { data: context };

    runtime().spawn(async move {
        let callback_context = callback_context;
        let response = message_interface::send_message(handle, message_type).await;
        let response = serde_json::to_string(&response).unwrap();
        let response = CString::new(response).unwrap();
        callback(response.as_ptr(), std::ptr::null(), callback_context.data);
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

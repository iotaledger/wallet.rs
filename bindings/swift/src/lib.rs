// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    events::types::{Event, WalletEventType},
    message_interface::{self, ManagerOptions, Message, WalletMessageHandler},
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

const INVALID_HANDLE_ERR: &str = "Invalid handle";

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

fn copy_error_message(dest: *mut c_char, src: Box<dyn std::fmt::Debug>, size: usize) {
    if dest.is_null() {
        return;
    }

    let error_message = CString::new(format!("{:?}", src)).unwrap();
    unsafe {
        error_message.as_ptr().copy_to(dest, size - 1);
    }
}

fn null_with_error_message(dest: *mut c_char, src: Box<dyn std::fmt::Debug>, size: usize) -> *mut IotaWalletHandle {
    copy_error_message(dest, src, size);
    std::ptr::null_mut()
}

/// # Safety
///
/// Ensure `error_buffer_size` doesn't exceed actual `error_buffer` size to avoid potential buffer overflow
#[no_mangle]
pub unsafe extern "C" fn iota_initialize(
    manager_options: *const c_char,
    error_buffer: *mut c_char,
    error_buffer_size: usize,
) -> *mut IotaWalletHandle {
    let manager_options = if manager_options.is_null() {
        None
    } else {
        let manager_options = CStr::from_ptr(manager_options);
        let manager_options = match manager_options.to_str() {
            Ok(manager_options) => manager_options,
            Err(e) => return null_with_error_message(error_buffer, Box::new(e), error_buffer_size),
        };
        let manager_options: ManagerOptions = match serde_json::from_str(manager_options) {
            Ok(manager_options) => manager_options,
            Err(e) => return null_with_error_message(error_buffer, Box::new(e), error_buffer_size),
        };
        Some(manager_options)
    };

    let handle = runtime().block_on(message_interface::create_message_handler(manager_options));

    match handle {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(e) => null_with_error_message(error_buffer, Box::new(e), error_buffer_size),
    }
}

/// # Safety
///
/// Does nothing if `handle` is null
#[no_mangle]
pub unsafe extern "C" fn iota_destroy(handle: *mut IotaWalletHandle) {
    if handle.is_null() {
        return;
    }

    Box::from_raw(handle);
}

/// # Safety
///
/// `callback` will be invoked from another thread so context must be safe to `Send` across threads
#[no_mangle]
pub unsafe extern "C" fn iota_send_message(
    handle: *mut IotaWalletHandle,
    message: *const c_char,
    callback: Callback,
    context: *mut c_void,
) {
    let (handle, c_message) = {
        if handle.is_null() || message.is_null() {
            let error = CString::new("Null error; either wallet handle or message is null").unwrap();
            return callback(std::ptr::null(), error.as_ptr(), context);
        }

        let handle = match handle.as_ref() {
            Some(handle) => handle,
            None => {
                let error = CString::new(INVALID_HANDLE_ERR).unwrap();
                return callback(std::ptr::null(), error.as_ptr(), context);
            }
        };

        (handle, CStr::from_ptr(message))
    };

    let message = c_message.to_str().unwrap();
    let message = match serde_json::from_str::<Message>(message) {
        Ok(message) => message,
        Err(e) => {
            let error = CString::new(format!("{:?}", e)).unwrap();
            return callback(std::ptr::null(), error.as_ptr(), context);
        }
    };

    let callback_context = CallbackContext { data: context };

    runtime().spawn(async move {
        let callback_context = callback_context;
        let response = message_interface::send_message(handle, message).await;
        let response = serde_json::to_string(&response).unwrap();
        let response = CString::new(response).unwrap();
        callback(response.as_ptr(), std::ptr::null(), callback_context.data);
    });
}

/// # Safety
///
/// `callback` will be invoked from another thread so context must be safe to `Send` across threads
#[no_mangle]
pub unsafe extern "C" fn iota_listen(
    handle: *mut IotaWalletHandle,
    event_types: *const c_char,
    callback: Callback,
    context: *mut c_void,
    error_buffer: *mut c_char,
    error_buffer_size: usize,
) -> i8 {
    let (handle, event_types) = {
        if handle.is_null() || event_types.is_null() {
            return -1;
        }

        let handle = match handle.as_ref() {
            Some(handle) => handle,
            None => {
                copy_error_message(error_buffer, Box::new(INVALID_HANDLE_ERR), error_buffer_size);
                return -1;
            }
        };

        let event_types = match CStr::from_ptr(event_types).to_str() {
            Ok(event_types) => event_types,
            Err(e) => {
                copy_error_message(error_buffer, Box::new(e), error_buffer_size);
                return -1;
            }
        };

        (handle, event_types)
    };

    let event_types: Vec<WalletEventType> = match serde_json::from_str(event_types) {
        Ok(event_types) => event_types,
        Err(e) => {
            copy_error_message(error_buffer, Box::new(e), error_buffer_size);
            return -1;
        }
    };

    let context = context as usize;

    runtime().spawn(handle.listen(event_types, move |event: &Event| {
        let message = serde_json::to_string(&event).unwrap();
        let message = CString::new(message).unwrap();
        let context = context as *mut c_void;
        callback(message.as_ptr(), std::ptr::null(), context);
    }));

    0
}

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

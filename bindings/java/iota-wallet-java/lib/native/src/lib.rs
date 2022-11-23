// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::TryFrom, sync::Mutex};

use iota_wallet::{
    events::types::WalletEventType,
    message_interface::{ManagerOptions, Message, WalletMessageHandler},
};
use jni::{
    objects::{JClass, JObject, JStaticMethodID, JString, JValue},
    signature::ReturnType,
    sys::{jclass, jobject, jobjectArray, jstring},
    JNIEnv, JavaVM,
};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};
use fern_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};

lazy_static! {
    static ref MESSAGE_HANDLER: Mutex<Option<WalletMessageHandler>> = Mutex::new(None);

    // Cache VM ref for callback
    static ref VM: Mutex<Option<JavaVM>> = Mutex::new(None);
    // Cache Method for unchecked method call
    static ref METHOD_CACHE: Mutex<Option<JStaticMethodID>> = Mutex::new(None);
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_initLogger(
    env: JNIEnv,
    _class: JClass,
    command: JString,
) {
    let config_builder: String = env.get_string(command).expect("Couldn't get java string!").into();

    let output_config: LoggerOutputConfigBuilder = serde_json::from_str(&config_builder).expect("invalid logger config");
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).expect("failed to init logger");
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_createMessageHandler(
    env: JNIEnv,
    // this is the class that owns our
    // static method. Not going to be
    // used, but still needs to have
    // an argument slot
    class: JClass,
    config: JString,
) {
    let manager_options: ManagerOptions = match env.get_string(config) {
        Ok(json_input) => match serde_json::from_str(&String::from(json_input)) {
            Ok(manager_options) => manager_options,
            Err(err) => {
                env.throw_new("java/lang/Exception", err.to_string()).unwrap();
                return;
            }
        }
        Err(err) => {
            env.throw_new("java/lang/Exception", err.to_string()).unwrap();
            return;
        }
    };

    match MESSAGE_HANDLER.lock() {
        Ok(mut message_handler_store) => match crate::block_on(iota_wallet::message_interface::create_message_handler(Some(manager_options))) {
            Ok(message_handler) => message_handler_store.replace(message_handler),
            Err(err) => {
                env.throw_new("java/lang/Exception", err.to_string()).unwrap();
                return;
            }
        },
        Err(err) => {
            env.throw_new("java/lang/Exception", err.to_string()).unwrap();
            return;
        }
    };
    VM.lock().unwrap().replace(env.get_java_vm().expect("GetJavaVm failed"));
    METHOD_CACHE.lock().unwrap().replace(
        env.get_static_method_id(
            class,
            "handleCallback",
            "(Ljava/lang/String;Lorg/iota/types/events/EventListener;)V",
        )
        .expect("find callback handler method"),
    );
}

// Destroy the required parts for messaging. Needs to call createMessageHandler again before resuming
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_destroyHandle(_env: JNIEnv, _class: JClass) {
    (*MESSAGE_HANDLER.lock().unwrap()) = None;
    (*VM.lock().unwrap()) = None;
    (*METHOD_CACHE.lock().unwrap()) = None;
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_sendMessage(
    env: JNIEnv,
    _class: JClass,
    command: JString,
) -> jstring {

    if env.exception_check().unwrap() {
        return std::ptr::null_mut();
    }

    let command: String = env
        .get_string(command)
        .expect("Couldn't get java string!")
        .into();

    let message = serde_json::from_str::<Message>(&command).unwrap();

    let (sender, mut receiver) = unbounded_channel();

    let guard = MESSAGE_HANDLER.lock().unwrap();
    crate::block_on(guard.as_ref().unwrap().handle(message, sender));

    let response = crate::block_on(receiver.recv()).unwrap();

    let output = env
        .new_string(serde_json::to_string(&response).unwrap())
        .expect("Couldn't create java string!");

    output.into_raw()
}

/// # Safety
///
/// Object::from_raw(callback) makes a java object from the callback.
/// This will crash if callback is java null
#[no_mangle]
pub unsafe extern "system" fn Java_org_iota_api_NativeApi_listen(
    env: JNIEnv,
    _class: jclass,
    events: jobjectArray,
    callback: jobject,
) -> jstring {
    let string_count = env.get_array_length(events).expect("Couldn't get array length!");
    let mut events_list: Vec<WalletEventType> = Vec::with_capacity(string_count.try_into().unwrap());

    for i in 0..string_count {
        let java_str: String = env
            .get_string(env.get_object_array_element(events, i).unwrap().into())
            .expect("Couldn't get java string!")
            .into();
        events_list.push(WalletEventType::try_from(&*java_str).expect("Unknon WalletEventType"));
    }

    // Allocate callback globaly to not lose the reference
    let global_obj = env
        .new_global_ref(JObject::from_raw(callback))
        .expect("Failed to make Global ref");

    let guard = MESSAGE_HANDLER.lock().unwrap();
    crate::block_on(guard.as_ref().unwrap().listen(events_list, move |e| {
        let ev_ser = serde_json::to_string(&e).expect("Failed to serialise event");

        // Grab env from VM
        let vm_guard = VM.lock().unwrap();
        let env = {
            let vm = vm_guard.as_ref().unwrap();
            let mut env = vm.get_env();
            if env.is_err() {
                env = vm.attach_current_thread().map(|e| *e);
            }
            env.expect("failed to get env")
        };

        let event = JValue::Object(*env.new_string(ev_ser).expect("Couldn't create event as string!")).to_jni();
        let cb = JValue::Object(global_obj.as_obj()).to_jni();

        env.call_static_method_unchecked(
            "org/iota/api/NativeApi",
            METHOD_CACHE.lock().unwrap().unwrap(),
            ReturnType::Object,
            &[event, cb],
        )
        .expect("Failed to call handleCallback");
    }));

    let output = env
        .new_string("{\"type\": \"success\", \"payload\": \"success\"}")
        .expect("Couldn't create java string!");

    output.into_raw()
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}
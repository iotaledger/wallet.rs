// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::TryFrom, sync::Mutex};

#[cfg(target_os = "android")]
use android_logger::Config;
use iota_wallet::{
    events::types::{Event, WalletEventType},
    message_interface::{create_message_handler, init_logger, ManagerOptions, Message, WalletMessageHandler},
};
use jni::{
    objects::{GlobalRef, JClass, JObject, JStaticMethodID, JString, JValue},
    signature::{Primitive, ReturnType},
    sys::{jclass, jobject, jobjectArray, jstring},
    JNIEnv, JavaVM,
};
#[cfg(target_os = "android")]
use log::{LevelFilter, error};
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    static ref MESSAGE_HANDLER: Mutex<Option<WalletMessageHandler>> = Mutex::new(None);

    // Cache VM ref for callback
    static ref VM: Mutex<Option<JavaVM>> = Mutex::new(None);

    // Cache Method for unchecked method call
    static ref METHOD_CACHE: Mutex<Option<JStaticMethodID>> = Mutex::new(None);
}

// A macro that checks if the result is an error and throws an exception if it is.
macro_rules! jni_err_assert {
    ($env:ident, $result:expr, $ret:expr ) => {{
        match $result {
            Err(err) => {
                throw_exception(&$env, err.to_string());
                return $ret;
            }
            Ok(res) => res,
        }
    }};
}

// Getting a string from the JNIEnv and then checking if it is an error. If it is an error,
// it throws an exception and returns the value of r. Otherwise returns a String.
macro_rules! string_from_jni {
    ($env:ident, $x:ident, $r:expr ) => {{
        let string = $env.get_string($x);
        String::from(jni_err_assert!($env, string, $r))
    }};
}

/// A macro that takes in a JNIEnv, a JString, a type, and a return value. It then gets the string from
/// the JNIEnv, and checks if it is an error. If it is an error, it throws an exception and returns
/// the value of r. Otherwise returns the unwrapped result.
macro_rules! jni_from_json_make {
    ($env:ident, $x:ident, $rtype:ident, $r:expr ) => {{
        let json = string_from_jni!($env, $x, $r);
        let jsonres: serde_json::error::Result<$rtype> = serde_json::from_str(&json);
        jni_err_assert!($env, jsonres, $r)
    }};
}

// This is a safety check to make sure that the JNIEnv is not in an exception state.
macro_rules! env_assert {
    ($env:ident, $r:expr ) => {{
        // Unwrap is not save, but nothing we can do if we are already in an error
        if $env.exception_check().unwrap() {
            return $r;
        }
    }};
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_initLogger(env: JNIEnv, _class: JClass, command: JString) {
    // This is a safety check to make sure that the JNIEnv is not in an exception state.
    env_assert!(env, ());
    let ret = init_logger(string_from_jni!(env, command, ()));
    jni_err_assert!(env, ret, ());
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
    env_assert!(env, ());

    #[cfg(target_os = "android")]
    android_logger::init_once(
        Config::default()
            .with_tag("lib_wallet")
            .with_max_level(LevelFilter::Off),
    );

    if let Ok(mut message_handler_store) = MESSAGE_HANDLER.lock() {
        message_handler_store.replace(jni_err_assert!(
            env,
            crate::block_on(create_message_handler(Some(jni_from_json_make!(
                env,
                config,
                ManagerOptions,
                ()
            ),))),
            ()
        ));
    };

    if let Ok(mut vm) = VM.lock() {
        vm.replace(jni_err_assert!(env, env.get_java_vm(), ()));
    }

    if let Ok(mut cache) = METHOD_CACHE.lock() {
        cache.replace(jni_err_assert!(
            env,
            env.get_static_method_id(
                class,
                "handleCallback",
                "(Ljava/lang/String;Lorg/iota/types/events/EventListener;)V",
            ),
            ()
        ));
    }
}

// Destroy the required parts for messaging. Needs to call createMessageHandler again before resuming
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_destroyHandle(env: JNIEnv, _class: JClass) {
    env_assert!(env, ());

    *jni_err_assert!(env, MESSAGE_HANDLER.lock(), ()) = None;
    *jni_err_assert!(env, VM.lock(), ()) = None;
    *jni_err_assert!(env, METHOD_CACHE.lock(), ()) = None;
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_sendMessage(
    env: JNIEnv,
    _class: JClass,
    command: JString,
) -> jstring {
    env_assert!(env, std::ptr::null_mut());

    let message = jni_from_json_make!(env, command, Message, std::ptr::null_mut());

    match MESSAGE_HANDLER.lock() {
        Ok(message_handler_store) => {
            match message_handler_store.as_ref() {
                Some(message_handler) => {
                    let res = crate::block_on(message_handler.send_message(message));
                    // We assume response is valid json from our own client
                    return make_jni_string(&env, serde_json::to_string(&res).unwrap());
                }
                _ => throw_nullpointer(&env, "Wallet not initialised."),
            }
        }
        Err(err) => throw_exception(&env, err.to_string()),
    };

    throw_nullpointer(&env, "Wallet not initialised.");
    std::ptr::null_mut()
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
    env_assert!(env, std::ptr::null_mut());

    let string_count = jni_err_assert!(env, env.get_array_length(events), std::ptr::null_mut());

    // Safe cast as we dont have that many events
    let mut events_list: Vec<WalletEventType> = Vec::with_capacity(string_count as usize);

    for i in 0..string_count {
        let arr_obj = jni_err_assert!(env, env.get_object_array_element(events, i), std::ptr::null_mut()).into();
        let java_str = string_from_jni!(env, arr_obj, std::ptr::null_mut());

        let event = jni_err_assert!(env, WalletEventType::try_from(&*java_str), std::ptr::null_mut());
        events_list.push(event);
    }

    // Allocate callback globaly to not lose the reference
    let global_obj = jni_err_assert!(
        env,
        env.new_global_ref(JObject::from_raw(callback)),
        std::ptr::null_mut()
    );

    // Allocate callback class to not lose the class in native threads
    let global_obj_class = jni_err_assert!(env, env.new_global_ref(JObject::from_raw(_class)), std::ptr::null_mut());

    match MESSAGE_HANDLER.lock() {
        Ok(message_handler_store) => match message_handler_store.as_ref() {
            Some(message_handler) => crate::block_on(message_handler.listen(events_list, move |e| {
                event_handle(global_obj_class.clone(), global_obj.clone(), e.clone());
            })),
            _ => throw_nullpointer(&env, "Wallet not initialised."),
        },
        Err(err) => throw_exception(&env, err.to_string()),
    };

    // exceptions return early, now we let the client know that we registered successfully
    make_jni_string(&env, "{\"type\": \"success\", \"payload\": \"success\"}".to_string())
}

unsafe fn event_handle(clazz: GlobalRef, callback_ref: GlobalRef, event: Event) {
    // Grab env from VM.
    // If we cant get a lock, we cant handle the event
    let vm_guard = match VM.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let vm = vm_guard
        .as_ref()
        .expect("Wallet not initialised, but an event was called");

    // Generate link back to the java env
    let env = vm
        .attach_current_thread()
        .expect("Failed to get Java env for event callback");

    // Make sure env is ok
    env_assert!(env, ());

    let ev_ser = jni_err_assert!(env, serde_json::to_string(&event), ());

    // Make the Jni object to send back
    let event = jni::sys::jvalue {
        l: make_jni_string(&env, ev_ser),
    };

    // Get a ref back to the callback we call on the java side
    let cb = JValue::Object(callback_ref.as_obj()).to_jni();
    let clazz = JClass::from_raw(*clazz.as_obj());

    // Call NativeApi, METHOD_CACHE assumed initialised if we received a VM
    let res = env.call_static_method_unchecked(
        clazz,
        METHOD_CACHE.lock().unwrap().unwrap(),
        ReturnType::Primitive(Primitive::Void),
        &[event, cb],
    );

    if let Err(e) = res {
        #[cfg(target_os = "android")]
        error!("Error calling method {}", e);
        #[cfg(not(target_os = "android"))]
        println!("Error calling method {}", e);
        jni_err_assert!(env, env.exception_clear(), ());
    };
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

fn make_jni_string(env: &JNIEnv, rust_str: std::string::String) -> jstring {
    match env.new_string(rust_str) {
        Ok(s) => s.into_raw(),
        Err(err) => {
            throw_exception(env, err.to_string());
            std::ptr::null_mut()
        }
    }
}

fn throw_exception(env: &JNIEnv, err: std::string::String) {
    throw("java/lang/Exception", env, &err)
}

fn throw_nullpointer(env: &JNIEnv, err: &str) {
    throw("java/lang/NullPointerException", env, err)
}

fn throw(exception: &str, env: &JNIEnv, err: &str) {
    env.throw_new(exception, err.to_string()).unwrap()
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;
use std::convert::TryFrom;

use iota_wallet::{
    events::types::WalletEventType,
    message_interface::{Message, ManagerOptions, WalletMessageHandler},
};
use jni::{
    objects::{JClass, JString},
    sys::{jstring, jlong, jobject, jobjectArray},
    JNIEnv, JavaVM
};
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

use lazy_static::lazy_static;

mod callback;
use callback::JavaCallback;

lazy_static! {
    static ref MESSAGE_HANDLER: Mutex<Option<WalletMessageHandler>> = Mutex::new(None);

    // Holds a list of registered callbacks ith their environment
    static ref EVENT_HANDLER: Mutex<Vec<JavaCallback>> = Mutex::new(Vec::new());
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_createMessageHandler(
    env: JNIEnv,
    // this is the class that owns our
    // static method. Not going to be
    // used, but still needs to have
    // an argument slot
    _class: JClass,
    config: JString,
) {
    let manager_options: ManagerOptions = {
        let input: String = env.get_string(config).expect("Couldn't get java string!").into();
        serde_json::from_str(&input).unwrap()
    };
    MESSAGE_HANDLER.lock().unwrap().replace(crate::block_on(iota_wallet::message_interface::create_message_handler(Some(manager_options))).unwrap());
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_destroyHandle(
    _env: JNIEnv,
    _class: JClass,
) {
    (*MESSAGE_HANDLER.lock().unwrap()) = None;
    (*EVENT_HANDLER.lock().unwrap()) = Vec::new();
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_sendMessage(
    env: JNIEnv,
    // this is the class that owns our
    // static method. Not going to be
    // used, but still needs to have
    // an argument slot
    _class: JClass,
    command: JString,
) -> jstring {
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

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub unsafe extern "system" fn Java_org_iota_api_NativeApi_listen(
    env: JNIEnv,
    _class: JClass,
    events: jobjectArray,
    id: jlong,
    callback: jobject,
) -> jstring {
    let r_id: i64 = id as i64;
    let string_count = env.get_array_length(events).expect("Couldn't get array length!");
    let mut events_list: Vec<WalletEventType> = Vec::with_capacity(string_count.try_into().unwrap());

    for i in 0..string_count {
        let java_str: String = env.get_string(env.get_object_array_element(events, i).unwrap().into()).expect("Couldn't get java string!").into();
        events_list.push(WalletEventType::try_from(&*java_str)
            .expect("Unknon WalletEventType"));
    }

    let stored_cb = JavaCallback::new(r_id, env, callback);
    let mut data = EVENT_HANDLER.lock().unwrap();
    let v: &mut Vec<JavaCallback> = &mut *data;
    v.push(stored_cb);

    let guard = MESSAGE_HANDLER.lock().unwrap();
    crate::block_on(guard.as_ref().unwrap().listen(events_list, move |e| {
        let ev_ser = serde_json::to_string(&e).expect("Failed to serialise event");
        dbg!(e);
        
        let data = EVENT_HANDLER.lock().unwrap();
        for jnicb in &*data {
            if jnicb.id == r_id {
                let vm = JavaVM::from_raw(jnicb.java_vm)
                    .expect("Couldn't create vm from pointer");
                let mut env = vm.get_env();
                if let Err(_) = env {
                    env = vm.attach_current_thread().map(|e|*e);
                }
                let env = env.expect("failed to get env");

                let output = env
                    .new_string(ev_ser)
                    .expect("Couldn't create event as string!");
                let class = env
                    .find_class("org/iota/api/NativeApi")
                    .expect("Failed to load the target class");
                env.call_static_method(class, "handleCallback", "(Ljava/lang/String;Lorg/iota/types/events/EventListener;)V", 
                    &[output.into(), jnicb.callback().into()]).expect("Failed to call handleCallback");
                
                break;
            }
        }
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

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::TryFrom, fs::File, io::Read, sync::Mutex};

use iota_wallet::{
    events::types::{TransactionProgressEvent, WalletEvent, WalletEventType},
    iota_client::api::PreparedTransactionDataDto,
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

lazy_static! {
    static ref MESSAGE_HANDLER: Mutex<Option<WalletMessageHandler>> = Mutex::new(None);

    // Cache VM ref for callback
    static ref VM: Mutex<Option<JavaVM>> = Mutex::new(None);
    // Cache Method for unchecked method call
    static ref METHOD_CACHE: Mutex<Option<JStaticMethodID>> = Mutex::new(None);
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
    let manager_options: ManagerOptions = {
        let input: String = env.get_string(config).expect("Couldn't get java string!").into();
        serde_json::from_str(&input).unwrap()
    };
    MESSAGE_HANDLER.lock().unwrap().replace(
        crate::block_on(iota_wallet::message_interface::create_message_handler(Some(
            manager_options,
        )))
        .unwrap(),
    );

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

#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_emit(_env: JNIEnv, _class: JClass) {
    let path = "src/main/res/prepared_transaction.json";
    let mut file = File::open(&path).unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();

    let prep = Box::new(serde_json::from_str::<PreparedTransactionDataDto>(&json).unwrap());
    let progress = TransactionProgressEvent::PreparedTransaction(prep);

    let event: WalletEvent = WalletEvent::TransactionProgress(progress);
    let message = Message::EmitTestEvent(event);

    let (sender, mut receiver) = unbounded_channel();
    let guard = MESSAGE_HANDLER.lock().unwrap();
    crate::block_on(guard.as_ref().unwrap().handle(message, sender));
    crate::block_on(receiver.recv()).unwrap();
}

// This keeps rust from "mangling" the name and making it unique for this crate.
#[no_mangle]
pub extern "system" fn Java_org_iota_api_NativeApi_sendMessage(
    env: JNIEnv,
    _class: JClass,
    command: JString,
) -> jstring {
    let command: String = env.get_string(command).expect("Couldn't get java string!").into();

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

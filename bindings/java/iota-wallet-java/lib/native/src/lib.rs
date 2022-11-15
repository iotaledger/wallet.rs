// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;

use iota_wallet::message_interface::{Message, ManagerOptions, WalletMessageHandler};
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::mpsc::unbounded_channel};

use lazy_static::lazy_static;

lazy_static! {
    static ref MESSAGE_HANDLER: Mutex<Option<WalletMessageHandler>> = Mutex::new(None);
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

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

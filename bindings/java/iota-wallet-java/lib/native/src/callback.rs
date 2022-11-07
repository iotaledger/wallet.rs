// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use jni::{
    objects::{JObject, GlobalRef},
    sys::{jstring, jlong, jobject, JavaVM as JniJavaVM},
    JNIEnv, JavaVM, AttachGuard
};

unsafe impl Send for JavaCallback {}
pub struct JavaCallback {
    pub java_vm: *mut JniJavaVM,
    pub id: i64,
    //this: GlobalRef,
}

/*
#[allow(dead_code)]
pub struct JniEnvHolder<'a> {
    env: Option<*mut JNIEnv<'a>>,
    callback: &'a JavaCallback,
}*/

impl JavaCallback {
    pub fn new(id: i64, env: JNIEnv) -> JavaCallback {
        let java_vm: *mut JniJavaVM = env.get_java_vm().expect("GetJavaVm failed").get_java_vm_pointer();
        JavaCallback {
            java_vm,
            id,
            //this: global_obj,
        }
    }
}
// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use jni::{
    sys::JavaVM,
    JNIEnv,
};

unsafe impl Send for JavaCallback {}
pub struct JavaCallback {
    pub java_vm: *mut JavaVM,
    pub id: i64,
}

impl JavaCallback {
    pub fn new(id: i64, env: JNIEnv) -> JavaCallback {
        let java_vm: *mut JavaVM = env.get_java_vm().expect("GetJavaVm failed").get_java_vm_pointer();
        JavaCallback {
            java_vm,
            id,
        }
    }
}
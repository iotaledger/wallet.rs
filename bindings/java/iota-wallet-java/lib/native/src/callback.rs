// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use jni::{
    objects::{JObject, GlobalRef},
    sys::{jobject, JavaVM as JniJavaVM},
    JNIEnv
};

unsafe impl Send for JavaCallback {}
pub struct JavaCallback {
    pub java_vm: *mut JniJavaVM,
    pub id: i64,
    this: GlobalRef,
}

#[allow(dead_code)]
pub struct JniEnvHolder<'a> {
    pub env: Option<JNIEnv<'a>>,
    callback: &'a JavaCallback,
}

impl JavaCallback {
    pub unsafe fn new(id: i64, env: JNIEnv, callback: jobject) -> JavaCallback {
        let global_obj = env.new_global_ref(JObject::from_raw(callback)).expect("Failed to make Global ref");
        let java_vm: *mut JniJavaVM = env.get_java_vm().expect("GetJavaVm failed").get_java_vm_pointer();
        JavaCallback {
            java_vm,
            id,
            this: global_obj,
        }
    }

    pub fn callback(&self) -> JObject {
        self.this.as_obj()
    }
}
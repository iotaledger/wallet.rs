use jni::objects::JClass;
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_org_iota_wallet_NativeAPI_verify_1link(_env: JNIEnv, _class: JClass) {
    dbg!("hello from rust!");
}
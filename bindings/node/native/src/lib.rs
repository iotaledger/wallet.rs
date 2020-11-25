use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::sync::Mutex;

use futures::{Future, FutureExt};
use iota_wallet::WalletError;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

mod classes;
use classes::*;

fn panic_to_response_message(panic: Box<dyn Any>) -> Result<String, WalletError> {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = backtrace::Backtrace::new();
    Ok(format!("{}\n\n{:?}", msg, current_backtrace))
}

pub async fn convert_async_panics<T, F: Future<Output = Result<T, WalletError>>>(
    f: impl FnOnce() -> F,
) -> Result<T, WalletError> {
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(WalletError::UnknownError(panic_to_response_message(panic)?)),
    }
}

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    static INSTANCE: OnceCell<Mutex<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

// Export the class
register_module!(mut m, {
    m.export_class::<JsAccountManager>("AccountManager")?;
    m.export_class::<JsAccount>("Account")?;
    m.export_class::<JsSyncedAccount>("SyncedAccount")?;
    m.export_class::<JsEventListener>("EventListener")?;
    Ok(())
});

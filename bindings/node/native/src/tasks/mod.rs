use std::any::Any;
use std::panic::AssertUnwindSafe;

use futures::{Future, FutureExt};
use iota_wallet::WalletError;

pub mod send;
pub mod sync;

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

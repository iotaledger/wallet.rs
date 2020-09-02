//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]
#![allow(unused_variables, dead_code)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The address module.
pub mod address;
/// The client module.
pub mod client;
/// The event module.
pub mod event;
/// The monitor module.
pub mod monitor;
/// The storage module.
pub mod storage;
/// The transaction module.
pub mod transaction;

pub use anyhow::Result;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex, MutexGuard};
use stronghold::Stronghold;

type GlobalStronghold = Arc<Mutex<Stronghold>>;
static STRONGHOLD_INSTANCE: OnceCell<GlobalStronghold> = OnceCell::new();

pub(crate) fn with_stronghold<T, F: FnOnce(MutexGuard<'static, Stronghold>) -> T>(cb: F) -> T {
  let stronghold = STRONGHOLD_INSTANCE.get_or_init(|| {
    let stronghold = Stronghold::new(storage::get_stronghold_snapshot_path());
    Arc::new(Mutex::new(stronghold))
  });
  cb(stronghold.lock().expect("failed to get stronghold lock"))
}

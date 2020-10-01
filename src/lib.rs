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
/// The message module.
pub mod message;
/// The monitor module.
pub mod monitor;
/// The storage module.
pub mod storage;

pub use anyhow::Result;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stronghold::Stronghold;

static STRONGHOLD_INSTANCE: OnceCell<Arc<Mutex<HashMap<PathBuf, Stronghold>>>> = OnceCell::new();

pub(crate) fn init_stronghold(stronghold_path: PathBuf, stronghold: Stronghold) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.insert(stronghold_path, stronghold);
}

pub(crate) fn remove_stronghold(stronghold_path: PathBuf) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.remove(&stronghold_path);
}

pub(crate) fn with_stronghold<T, F: FnOnce(&Stronghold) -> T>(cb: F) -> T {
    with_stronghold_from_path(&crate::storage::get_stronghold_snapshot_path(), cb)
}

pub(crate) fn with_stronghold_from_path<T, F: FnOnce(&Stronghold) -> T>(
    path: &PathBuf,
    cb: F,
) -> T {
    let stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    if let Some(stronghold) = stronghold_map.get(path) {
        cb(stronghold)
    } else {
        panic!("should initialize stronghold instance before using it")
    }
}

#[cfg(test)]
mod test_utils {
    use super::account_manager::AccountManager;
    use once_cell::sync::OnceCell;

    static MANAGER_INSTANCE: OnceCell<AccountManager> = OnceCell::new();
    pub fn get_account_manager() -> &'static AccountManager {
        MANAGER_INSTANCE.get_or_init(|| {
            let manager = AccountManager::new();
            manager.set_stronghold_password("password").unwrap();
            manager
        })
    }
}

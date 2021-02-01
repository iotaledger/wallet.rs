pub mod classes;
pub mod types;

use pyo3::prelude::*;
use types::*;

/// A Python module implemented in Rust.
#[pymodule]
fn iota_client(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AccountInitialiser>()?;
    m.add_class::<AccountHandle>()?;
    m.add_class::<SyncedAccount>()?;
    m.add_class::<AccountSynchronizer>()?;
    m.add_class::<Transfer>()?;
    m.add_class::<AccountManager>()?;
    Ok(())
}

// #[pyclass]
// pub struct AccountManager {
//     pub account_manager: RustAccountManager,
// }

// #[pyclass]
// pub struct AccountInitialiser {
//     pub account_initialiser: Option<RustAccountInitialiser>,
// }

// #[pyclass]
// pub struct AccountHandle {
//     pub account_handle: RustAccountHandle,
// }

// #[pyclass]
// pub struct SyncedAccount {
//     pub synced_account: RustSyncedAccount,
// }

// #[pyclass]
// pub struct AccountSynchronizer {
//     pub account_synchronizer: Option<RustAccountSynchronizer>,
// }

// #[pyclass]
// #[derive(Debug, Clone)]
// pub struct Transfer {
//     pub transfer: RustTransfer,
// }

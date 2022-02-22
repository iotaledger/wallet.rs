// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::path::PathBuf;

pub use iota_wallet::account_manager::AccountManager;
use iota_wallet::storage::constants::DEFAULT_STORAGE_FOLDER;

// todo use from main Rust crate
#[derive(Default, Deserialize)]
pub struct ManagerOptions {
    #[serde(rename = "storagePath", default = "default_storage_path")]
    pub storage_path: PathBuf,
    #[serde(rename = "storagePassword")]
    pub storage_password: Option<String>,
    #[serde(rename = "outputConsolidationThreshold")]
    pub output_consolidation_threshold: Option<usize>,
    #[serde(
        rename = "automaticOutputConsolidation",
        default = "default_automatic_output_consolidation"
    )]
    pub automatic_output_consolidation: bool,
    #[serde(rename = "syncSpentOutputs", default)]
    pub sync_spent_outputs: bool,
    #[serde(rename = "persistEvents", default)]
    pub persist_events: bool,
    #[serde(rename = "allowCreateMultipleEmptyAccounts", default)]
    pub allow_create_multiple_empty_accounts: bool,
    #[serde(rename = "skipPolling", default = "default_skip_polling")]
    pub skip_polling: bool,
    #[serde(rename = "pollingInterval")]
    pub polling_interval: Option<u64>,
}

fn default_storage_path() -> PathBuf {
    DEFAULT_STORAGE_FOLDER.into()
}

fn default_automatic_output_consolidation() -> bool {
    true
}

fn default_skip_polling() -> bool {
    false
}

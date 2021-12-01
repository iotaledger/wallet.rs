// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::types::{address::AccountAddress, OutputKind};

use iota_client::bee_message::{output::OutputId, payload::indexation::IndexationPayload};
use serde::{Deserialize, Serialize};

/// Options for value transfers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferOptions {
    #[serde(rename = "remainderValueStrategy", default)]
    pub remainder_value_strategy: RemainderValueStrategy,
    #[serde(rename = "remainderOutputKind", default)]
    pub remainder_output_kind: Option<OutputKind>,
    pub indexation: Option<IndexationPayload>,
    #[serde(rename = "skipSync", default)]
    pub skip_sync: bool,
    #[serde(rename = "customInputs", default)]
    pub custom_inputs: Option<Vec<OutputId>>,
}

// clearer to have it here in transfer.rs or also mvoe it into the types folder?

/// An output for a value transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOutput {
    pub address: String,
    pub amount: u64,
    #[serde(rename = "outputKind", default)]
    pub output_kind: Option<OutputKind>,
}

#[allow(clippy::enum_variant_names)]
/// The strategy to use for the remainder value management when sending funds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum RemainderValueStrategy {
    /// Keep the remainder value on the source address.
    ReuseAddress,
    /// Move the remainder value to a change address.
    ChangeAddress,
    /// Move the remainder value to any specified address.
    CustomAddress(AccountAddress),
}

impl Default for RemainderValueStrategy {
    fn default() -> Self {
        // ChangeAddress is the default because it's better for privacy than reusing an address.
        Self::ChangeAddress
    }
}

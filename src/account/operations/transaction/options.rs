// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    output::OutputId,
    payload::{dto::TaggedDataPayloadDto, tagged_data::TaggedDataPayload},
    DtoError,
};
use serde::{Deserialize, Serialize};

use crate::account::types::address::AccountAddress;

/// Options for transactions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionOptions {
    #[serde(rename = "remainderValueStrategy", default)]
    pub remainder_value_strategy: RemainderValueStrategy,
    #[serde(rename = "taggedDataPayload", default)]
    pub tagged_data_payload: Option<TaggedDataPayload>,
    // If custom inputs are provided only they are used. If also other additional inputs should be used,
    // `mandatory_inputs` should be used instead.
    #[serde(rename = "customInputs", default)]
    pub custom_inputs: Option<Vec<OutputId>>,
    #[serde(rename = "mandatoryInputs", default)]
    pub mandatory_inputs: Option<Vec<OutputId>>,
    #[serde(rename = "allowBurning", default)]
    pub allow_burning: bool,
    pub note: Option<String>,
}

impl TransactionOptions {
    /// Conversion from [`TransactionOptionsDto`] to [`TransactionOptions`].
    pub fn try_from_dto(value: &TransactionOptionsDto) -> Result<Self, DtoError> {
        Ok(TransactionOptions {
            remainder_value_strategy: value.remainder_value_strategy.clone(),
            tagged_data_payload: match &value.tagged_data_payload {
                Some(tagged_data_payload_dto) => Some(TaggedDataPayload::try_from(tagged_data_payload_dto)?),
                None => None,
            },
            custom_inputs: value.custom_inputs.clone(),
            mandatory_inputs: value.mandatory_inputs.clone(),
            allow_burning: value.allow_burning,
            note: value.note.clone(),
        })
    }
}

/// Dto for transaction options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionOptionsDto {
    #[serde(rename = "remainderValueStrategy", default)]
    pub remainder_value_strategy: RemainderValueStrategy,
    #[serde(rename = "taggedDataPayload", default)]
    pub tagged_data_payload: Option<TaggedDataPayloadDto>,
    // If custom inputs are provided only they are used. If also other additional inputs should be used,
    // `mandatory_inputs` should be used instead.
    #[serde(rename = "customInputs", default)]
    pub custom_inputs: Option<Vec<OutputId>>,
    #[serde(rename = "mandatoryInputs", default)]
    pub mandatory_inputs: Option<Vec<OutputId>>,
    #[serde(rename = "allowBurning", default)]
    pub allow_burning: bool,
    pub note: Option<String>,
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
        Self::ReuseAddress
    }
}

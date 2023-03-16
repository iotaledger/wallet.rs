// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota_client::{
    api::PreparedTransactionDataDto,
    api_types::core::response::OutputWithMetadataResponse,
    block::payload::transaction::{dto::TransactionPayloadDto, TransactionId},
};
use serde::{Deserialize, Serialize, Serializer};

use crate::account::types::{address::AddressWrapper, InclusionState, OutputDataDto};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Associated account index.
    #[serde(rename = "accountIndex")]
    pub account_index: u32,
    /// The event
    pub event: WalletEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalletEvent {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ledger_nano")))]
    LedgerAddressGeneration(AddressData),
    NewOutput(Box<NewOutputEvent>),
    SpentOutput(Box<SpentOutputEvent>),
    TransactionInclusion(TransactionInclusionEvent),
    TransactionProgress(TransactionProgressEvent),
}

impl Serialize for WalletEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum WalletEvent_<'a> {
            T0,
            T1(&'a AddressData),
            T2(&'a NewOutputEvent),
            T3(&'a SpentOutputEvent),
            T4(&'a TransactionInclusionEvent),
            T5(&'a TransactionProgressEvent),
        }
        #[derive(Serialize)]
        struct TypedWalletEvent_<'a> {
            #[serde(rename = "type")]
            kind: u8,
            #[serde(flatten)]
            event: WalletEvent_<'a>,
        }
        let event = match self {
            Self::ConsolidationRequired => TypedWalletEvent_ {
                kind: 0,
                event: WalletEvent_::T0,
            },
            Self::LedgerAddressGeneration(e) => TypedWalletEvent_ {
                kind: 1,
                event: WalletEvent_::T1(e),
            },
            Self::NewOutput(e) => TypedWalletEvent_ {
                kind: 2,
                event: WalletEvent_::T2(e),
            },
            Self::SpentOutput(e) => TypedWalletEvent_ {
                kind: 3,
                event: WalletEvent_::T3(e),
            },
            Self::TransactionInclusion(e) => TypedWalletEvent_ {
                kind: 4,
                event: WalletEvent_::T4(e),
            },
            Self::TransactionProgress(e) => TypedWalletEvent_ {
                kind: 5,
                event: WalletEvent_::T5(e),
            },
        };
        event.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WalletEvent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        Ok(
            match value
                .get("type")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| serde::de::Error::custom("invalid event type"))? as u8
            {
                0 => Self::ConsolidationRequired,
                1 => Self::LedgerAddressGeneration(AddressData::deserialize(value).map_err(|e| {
                    serde::de::Error::custom(format!("cannot deserialize LedgerAddressGeneration: {e}"))
                })?),
                2 => {
                    Self::NewOutput(Box::new(NewOutputEvent::deserialize(value).map_err(|e| {
                        serde::de::Error::custom(format!("cannot deserialize NewOutput: {e}"))
                    })?))
                }
                3 => {
                    Self::SpentOutput(Box::new(SpentOutputEvent::deserialize(value).map_err(|e| {
                        serde::de::Error::custom(format!("cannot deserialize SpentOutput: {e}"))
                    })?))
                }
                4 => {
                    Self::TransactionInclusion(TransactionInclusionEvent::deserialize(value).map_err(|e| {
                        serde::de::Error::custom(format!("cannot deserialize TransactionInclusion: {e}"))
                    })?)
                }
                5 => Self::TransactionProgress(TransactionProgressEvent::deserialize(value).map_err(|e| {
                    serde::de::Error::custom(format!("cannot deserialize TransactionProgressEvent: {e}"))
                })?),
                _ => return Err(serde::de::Error::custom("invalid event type")),
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalletEventType {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ledger_nano")))]
    LedgerAddressGeneration,
    NewOutput,
    SpentOutput,
    TransactionInclusion,
    TransactionProgress,
}

impl TryFrom<&str> for WalletEventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let event_type = match value {
            "ConsolidationRequired" => Self::ConsolidationRequired,
            #[cfg(feature = "ledger_nano")]
            "LedgerAddressGeneration" => Self::LedgerAddressGeneration,
            "NewOutput" => Self::NewOutput,
            "SpentOutput" => Self::SpentOutput,
            "TransactionInclusion" => Self::TransactionInclusion,
            "TransactionProgress" => Self::TransactionProgress,
            _ => return Err(format!("invalid event type {value}")),
        };
        Ok(event_type)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewOutputEvent {
    /// The new output.
    pub output: OutputDataDto,
    /// The transaction that created the output. Might be pruned and not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionPayloadDto>,
    /// The inputs for the transaction that created the output. Might be pruned and not available.
    #[serde(rename = "transactionInputs", skip_serializing_if = "Option::is_none")]
    pub transaction_inputs: Option<Vec<OutputWithMetadataResponse>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpentOutputEvent {
    /// The spent output.
    pub output: OutputDataDto,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TransactionInclusionEvent {
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    #[serde(rename = "inclusionState")]
    pub inclusion_state: InclusionState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionProgressEvent {
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(Box<PreparedTransactionDataDto>),
    /// Prepared transaction essence hash hex encoded, required for blindsigning with a ledger nano
    PreparedTransactionEssenceHash(String),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPow,
    /// Broadcasting.
    Broadcasting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AddressConsolidationNeeded {
    /// The associated address.
    #[serde(with = "crate::account::types::address_serde")]
    pub address: AddressWrapper,
}

/// Address event data.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq, Eq, Hash)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: String,
}

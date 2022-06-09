// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota_client::{api::PreparedTransactionDataDto, bee_block::payload::transaction::TransactionId};
use serde::{Deserialize, Serialize};

use crate::{account::types::InclusionState, message_interface::dtos::OutputDataDto};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Associated account index.
    #[serde(rename = "accountIndex")]
    pub account_index: u32,
    /// The event
    pub event: WalletEvent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WalletEvent {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    LedgerAddressGeneration(AddressData),
    NewOutput(NewOutputEvent),
    SpentOutput(SpentOutputEvent),
    TransactionInclusion(TransactionInclusionEvent),
    TransactionProgress(TransactionProgressEvent),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalletEventType {
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
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
            "ConsolidationRequired" => WalletEventType::ConsolidationRequired,
            #[cfg(feature = "ledger_nano")]
            "LedgerAddressGeneration" => WalletEventType::LedgerAddressGeneration,
            "NewOutput" => WalletEventType::NewOutput,
            "SpentOutput" => WalletEventType::SpentOutput,
            "TransactionInclusion" => WalletEventType::TransactionInclusion,
            "TransactionProgress" => WalletEventType::TransactionProgress,
            _ => return Err(format!("invalid event type {}", value)),
        };
        Ok(event_type)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewOutputEvent {
    /// The new output.
    pub output: OutputDataDto,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpentOutputEvent {
    /// The spent output.
    pub output: OutputDataDto,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TransactionInclusionEvent {
    pub transaction_id: TransactionId,
    pub inclusion_state: InclusionState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionProgressEvent {
    /// Syncing account.
    SyncingAccount,
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(Box<PreparedTransactionDataDto>),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPoW,
    /// Broadcasting.
    Broadcasting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AddressConsolidationNeeded {
    /// The associated address.
    pub address: String,
}

/// Address event data.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq, Eq, Hash)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: String,
}

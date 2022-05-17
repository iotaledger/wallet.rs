// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::Getters;
use iota_client::bee_block::payload::transaction::TransactionId;
use serde::{Deserialize, Serialize};

use crate::account::types::{address::AddressWrapper, InclusionState};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Event {
    /// Associated account index.
    #[serde(rename = "accountIndex")]
    pub account_index: u32,
    /// The event
    pub event: WalletEvent,
}

// do we want an event for transaction confirmation or if it failed?
// event for new detected outputs?

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WalletEvent {
    BalanceChange(BalanceChangeEvent),
    TransactionInclusion(TransactionInclusionEvent),
    TransferProgress(TransferProgressEvent),
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    LedgerAddressGeneration(AddressData),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WalletEventType {
    BalanceChange,
    TransactionInclusion,
    TransferProgress,
    ConsolidationRequired,
    #[cfg(feature = "ledger_nano")]
    LedgerAddressGeneration,
}

impl TryFrom<&str> for WalletEventType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let event_type = match value {
            "BalanceChange" => WalletEventType::BalanceChange,
            "TransactionInclusion" => WalletEventType::TransactionInclusion,
            "TransferProgress" => WalletEventType::TransferProgress,
            "ConsolidationRequired" => WalletEventType::ConsolidationRequired,
            #[cfg(feature = "ledger_nano")]
            "LedgerAddressGeneration" => WalletEventType::LedgerAddressGeneration,
            _ => return Err(format!("invalid event type {}", value)),
        };
        Ok(event_type)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BalanceChangeEvent {
    /// The address.
    pub address: AddressWrapper,
    /// The balance change data.
    pub balance_change: i64,
    /// Total account balance
    pub new_balance: u64,
    // the output/transaction?
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TransactionInclusionEvent {
    pub transaction_id: TransactionId,
    pub inclusion_state: InclusionState,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransferProgressEvent {
    /// Syncing account.
    SyncingAccount,
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(PreparedTransactionEventData),
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPoW,
    /// Broadcasting.
    Broadcasting,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AddressConsolidationNeeded {
    /// The associated address.
    #[serde(with = "crate::account::types::address_serde")]
    pub address: AddressWrapper,
}

/// Address event data.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: String,
}

/// Prepared transaction event data.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct PreparedTransactionEventData {
    /// Transaction inputs.
    pub inputs: Vec<TransactionIO>,
    /// Transaction outputs.
    pub outputs: Vec<TransactionIO>,
    /// The indexation data.
    pub data: Option<String>,
}

/// Input or output data for PreparedTransactionData
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TransactionIO {
    /// Address
    pub address: String,
    /// Amount
    pub amount: u64,
    /// Remainder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remainder: Option<bool>,
}

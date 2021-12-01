// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::types::{address::AddressWrapper, InclusionState};

use getset::Getters;
use iota_client::bee_message::payload::transaction::TransactionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Event {
    /// Associated account index.
    #[serde(rename = "accountIndex")]
    pub account_index: usize,
    /// The event
    pub event: WalletEvent,
}

// do we want an event for transaction confirmation or if it failed?
// event for new detected outputs?

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalletEvent {
    BalanceChange(BalanceChangeEvent),
    TransactionInclusion(TransactionInclusionEvent),
    TransferProgress(TransferProgressEvent),
    ConsolidationRequired,
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    LedgerAddressGeneration(AddressData),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalletEventType {
    BalanceChange,
    TransactionInclusion,
    TransferProgress,
    ConsolidationRequired,
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    LedgerAddressGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BalanceChangeEvent {
    /// The address.
    pub address: AddressWrapper,
    /// The balance change data.
    pub balance_change: i64,
    /// Total account balance
    pub new_balance: u64,
    // the output/transaction?
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionInclusionEvent {
    pub transaction_id: TransactionId,
    pub inclusion_state: InclusionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransferProgressEvent {
    /// Syncing account.
    SyncingAccount,
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress(AddressData),
    /// Prepared transaction.
    PreparedTransaction(PreparedTransactionData),
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

/// Prepared transaction event data.
#[derive(Debug, Clone, Getters, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[getset(get = "pub")]
pub struct PreparedTransactionData {
    /// Transaction inputs.
    pub inputs: Vec<TransactionIO>,
    /// Transaction outputs.
    pub outputs: Vec<TransactionIO>,
    /// The indexation data.
    pub data: Option<String>,
}

/// Input or output data for PreparedTransactionData
#[derive(Debug, Clone, Getters, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

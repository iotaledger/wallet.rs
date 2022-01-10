// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type TransactionId = String;
type MessageId = String;
type AddressWrapper = String;
type OutputId = String;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::account::schema) enum OutputKind {
    /// SignatureLockedSingle output.
    SignatureLockedSingle,
    /// Dust allowance output.
    SignatureLockedDustAllowance,
    /// Treasury output.
    Treasury,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::account::schema) struct AddressOutput {
    /// Transaction ID of the output
    pub(in crate::account::schema) transaction_id: TransactionId,
    /// Message ID of the output
    pub message_id: MessageId,
    /// Output index.
    pub(in crate::account::schema) index: u16,
    /// Output amount.
    pub(in crate::account::schema) amount: u64,
    /// Spend status of the output,
    pub(in crate::account::schema) is_spent: bool,
    /// Associated address.
    pub(in crate::account::schema) address: AddressWrapper,
    /// Output kind.
    pub(in crate::account::schema) kind: OutputKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::account::schema) struct Address {
    pub(in crate::account::schema) address: AddressWrapper,
    pub(in crate::account::schema) balance: u64,
    pub(in crate::account::schema) key_index: usize,
    pub(in crate::account::schema) internal: bool,
    pub(in crate::account::schema) outputs: HashMap<OutputId, AddressOutput>,
}

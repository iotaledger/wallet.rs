// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::SignatureLockedSingleOutput;
use iota_wallet::message::{
    MessageMigratedFundsEntry as MigratedFundsEntryRust, MessageReceiptPayload as ReceiptPayloadRust,
};
use std::fmt::{Display, Formatter};

pub struct ReceiptPayload {
    payload: ReceiptPayloadRust,
}

impl From<ReceiptPayloadRust> for ReceiptPayload {
    fn from(payload: ReceiptPayloadRust) -> Self {
        Self { payload }
    }
}

impl ReceiptPayload {
    pub fn migrated_at(&self) -> u32 {
        *self.payload.migrated_at()
    }

    pub fn last(&self) -> bool {
        self.payload.last()
    }

    pub fn funds(&self) -> Vec<MigratedFundsEntry> {
        self.payload
            .funds()
            .into_iter()
            .map(|m| MigratedFundsEntry { payload: m.clone() })
            .collect()
    }
}

impl Display for ReceiptPayload {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.payload)
    }
}

pub struct MigratedFundsEntry {
    payload: MigratedFundsEntryRust,
}

impl MigratedFundsEntry {
    pub fn tail_transaction_hash(&self) -> String {
        self.payload.tail_transaction_hash().to_string()
    }

    pub fn output(&self) -> SignatureLockedSingleOutput {
        SignatureLockedSingleOutput::from_rust(self.payload.output().clone())
    }
}

impl Display for MigratedFundsEntry {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.payload)
    }
}

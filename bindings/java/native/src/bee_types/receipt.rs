// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::message::{ReceiptPayload as ReceiptPayloadRust, SignatureLockedSingleOutput};

use iota::MigratedFundsEntry as MigratedFundsEntryRust;

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
        self.payload.migrated_at()
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

    pub fn amount(&self) -> u64 {
        self.payload.amount()
    }
}

pub struct MigratedFundsEntry {
    payload: MigratedFundsEntryRust,
}

impl MigratedFundsEntry {
    pub fn tail_transaction_hash(&self) -> Vec<u8> {
        self.payload.tail_transaction_hash().into_iter().map(|h| *h).collect()
    }

    pub fn output(&self) -> SignatureLockedSingleOutput {
        self.payload.output().clone()
    }
}

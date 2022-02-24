// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    iota_client::bee_message::output::Output, message::TreasuryTransactionPayload as TreasuryTransactionPayloadRust,
};

pub struct TreasuryPayload {
    payload: TreasuryTransactionPayloadRust,
}

impl From<TreasuryTransactionPayloadRust> for TreasuryPayload {
    fn from(payload: TreasuryTransactionPayloadRust) -> Self {
        Self { payload }
    }
}

impl TreasuryPayload {
    pub fn output(&self) -> u64 {
        if let Output::Treasury(payload) = self.payload.output() {
            return payload.amount();
        }
        unreachable!()
    }
}

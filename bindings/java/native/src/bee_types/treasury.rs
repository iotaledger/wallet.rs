// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::message::{MessageId, TreasuryTransactionPayload as TreasuryTransactionPayloadRust};

use iota::{Input, Output};

pub struct TreasuryTransactionPayload {
    payload: TreasuryTransactionPayloadRust,
}

impl From<TreasuryTransactionPayloadRust> for TreasuryTransactionPayload {
    fn from(payload: TreasuryTransactionPayloadRust) -> Self {
        Self { payload }
    }
}

impl TreasuryTransactionPayload {
    pub fn input(&self) -> MessageId {
        if let Input::Treasury(payload) = self.payload.input() {
            return payload.message_id().clone();
        }
        unreachable!()
    }

    pub fn output(&self) -> u64 {
        if let Output::Treasury(payload) = self.payload.output() {
            return payload.amount();
        }
        unreachable!()
    }
}

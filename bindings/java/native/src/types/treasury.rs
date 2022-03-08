// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    iota_client::bee_message::{
        output::Output,
        input::Input,
    },
    message::TreasuryTransactionPayload as TreasuryTransactionPayloadRust,
};

use crate::{TreasuryOutput, TreasuryInput};

pub struct TreasuryPayload {
    payload: TreasuryTransactionPayloadRust,
}

impl From<TreasuryTransactionPayloadRust> for TreasuryPayload {
    fn from(payload: TreasuryTransactionPayloadRust) -> Self {
        Self { payload }
    }
}

impl TreasuryPayload {
    pub fn output(&self) -> TreasuryOutput {
        if let Output::Treasury(payload) = self.payload.output() {
            return payload.clone().into();
        }
        unreachable!()
    }

    pub fn input(&self) -> TreasuryInput {
        if let Input::Treasury(payload) = self.payload.input() {
            return (*payload).into();
        }
        unreachable!()
    }
}

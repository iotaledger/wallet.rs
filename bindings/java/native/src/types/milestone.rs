// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Result;

use iota_wallet::message::{
    MessageId, MessageMilestonePayload as MilestonePayloadRust,
    MessageMilestonePayloadEssence as RustMilestonePayloadEssence,
};

use anyhow::anyhow;

pub struct MilestonePayload {
    payload: MilestonePayloadRust,
}

impl MilestonePayload {
    pub async fn new(essence: RustMilestonePayloadEssence, signatures: Vec<Box<[u8]>>) -> Result<MilestonePayload> {
        let res = MilestonePayloadRust::new(essence, signatures).await;
        match res {
            Ok(index) => Ok(MilestonePayload { payload: index }),
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }

    pub fn essence(&self) -> MilestonePayloadEssence {
        let ess_ref = self.payload.essence();

        MilestonePayloadEssence {
            essence: ess_ref.clone(),
        }
    }

    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
        self.payload.signatures()
    }
}
pub struct MilestonePayloadEssence {
    essence: RustMilestonePayloadEssence,
}

impl MilestonePayloadEssence {
    pub fn index(&self) -> u32 {
        *self.essence.index()
    }

    pub fn timestamp(&self) -> u64 {
        self.essence.timestamp()
    }

    pub fn parents(&self) -> Vec<MessageId> {
        self.essence.parents().iter().map(|e| e.clone()).collect()
    }

    pub fn merkle_proof(&self) -> Vec<u8> {
        self.essence.merkle_proof().to_vec()
    }
}

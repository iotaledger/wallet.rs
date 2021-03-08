use anyhow::anyhow;
use crate::Result;
use std::convert::TryInto;

use iota_wallet::{
    message::{
        MilestonePayload as MilestonePayloadRust,
        MessageId,
    }
};

use iota::{
    MilestonePayloadEssence as RustMilestonePayloadEssence,
};

pub struct MilestonePayload {
    payload: MilestonePayloadRust,
}

impl MilestonePayload {
    pub fn new_with(essence: RustMilestonePayloadEssence, signatures: Vec<Box<[u8]>>) -> Result<MilestonePayload> {
        let index = MilestonePayloadRust::new(essence, signatures);
        Ok(MilestonePayload {
                payload: index
            }
        )
    }

    pub fn id(&self) -> String {
        self.payload.id().to_string()
    }

    pub fn essence(&self) -> MilestonePayloadEssence {
        let ess_ref = self.payload.essence();
        MilestonePayloadEssence {
            essence: RustMilestonePayloadEssence::new(
                ess_ref.index(),
                ess_ref.timestamp(),
                ess_ref.parents().to_vec(),
                ess_ref.merkle_proof().try_into().unwrap(),
                ess_ref.public_keys().to_owned(),
            )
        }
    }

    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
        self.payload.signatures()
    }

    pub fn validate(&self, applicable_public_keys: &[String], min_threshold: usize) -> bool {
        let res = self.payload.validate(applicable_public_keys, min_threshold);
        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
pub struct MilestonePayloadEssence {
    essence: RustMilestonePayloadEssence,
}

impl MilestonePayloadEssence {
    pub fn index(&self) -> u32 {
        self.essence.index()
    }

    pub fn timestamp(&self) -> u64 {
        self.essence.timestamp()
    }

    pub fn parents(&self) -> Vec<MessageId> {
        (*self.essence.parents()).to_vec()
    }

    pub fn merkle_proof(&self) -> Vec<u8> {
        self.essence.merkle_proof().to_vec()
    }
}
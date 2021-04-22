// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::message::{
    MessageId,
    MessageMilestonePayloadEssence as MilestonePayloadEssenceRust,
    MessagePayload as MessagePayloadRust,
};

use crate::ReceiptPayload;

pub struct MilestonePayload {
    essence: MilestonePayloadEssenceRust,
    signatures: Vec<Box<[u8]>>,
}

impl MilestonePayload {
    pub fn new(essence: MilestonePayloadEssenceRust, signatures: Vec<Box<[u8]>>) -> MilestonePayload {
        MilestonePayload {
            essence,
            signatures
        }
    }

    pub fn essence(&self) -> MilestonePayloadEssence {
        MilestonePayloadEssence {
            essence: self.essence.clone(),
        }
    }

    pub fn signatures(&self) -> Vec<MilestoneSignature> {
        // Vec of vec, or vec of box isnt implemented as a generatable type
        self.signatures.clone()
            .iter()
            .map(|signature| MilestoneSignature { 
                signature: (*signature).to_vec()
            })
            .collect()
    }
}

pub struct MilestoneSignature {
    signature: Vec<u8>,
}

impl MilestoneSignature {
    pub fn get_signature(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

pub struct MilestonePayloadEssence {
    essence: MilestonePayloadEssenceRust,
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

    pub fn next_pow_score(&self) -> u32 {
        self.essence.next_pow_score()
    }

    pub fn next_pow_score_milestone(&self) -> u32 {
        self.essence.next_pow_score_milestone_index()
    }

    pub fn public_keys(&self) -> Vec<PublicKey> {
        // Vec of vec isnt implemented as a generatable type
        self.essence.public_keys().iter().map(|key| PublicKey {public_key: key.to_vec()}).collect()
    }

    pub fn receipt(&self) -> Option<ReceiptPayload> {
        let option = self.essence.receipt();
        if let Some(payload) = option {
            if let MessagePayloadRust::Receipt(receipt) = payload {
                return Some((*receipt.clone()).into())
            }
        }

        None
    }
}

pub struct PublicKey {
    public_key: Vec<u8>,
}

impl PublicKey {
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }
}

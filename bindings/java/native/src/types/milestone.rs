// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::message::{
    MessageId, MessageMilestonePayloadEssence as MilestonePayloadEssenceRust, MessagePayload as MessagePayloadRust,
};

use iota_client::crypto::signatures::ed25519::{
    PublicKey as RustPublicKey, Signature as RustSignature
};

use std::convert::TryInto;

use crate::{
    ReceiptPayload,
    Result
};

const SECRET_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

#[derive(PartialEq, Debug)]
pub struct MilestonePayload {
    essence: MilestonePayloadEssenceRust,
    signatures: Vec<Box<[u8]>>,
}

impl MilestonePayload {
    pub fn new(essence: MilestonePayloadEssenceRust, signatures: Vec<Box<[u8]>>) -> MilestonePayload {
        MilestonePayload { essence, signatures }
    }

    pub fn essence(&self) -> MilestonePayloadEssence {
        MilestonePayloadEssence {
            essence: self.essence.clone(),
        }
    }

    pub fn signatures(&self) -> Vec<MilestoneSignature> {
        // Vec of vec, or vec of box isnt implemented as a generatable type
        self.signatures
            .clone()
            .iter()
            .map(|signature| MilestoneSignature {
                signature: (*signature).to_vec(),
            })
            .collect()
    }
}

impl core::fmt::Display for MilestonePayload {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "essence={:?} signatures=({:?})", self.essence, self.signatures)
    }
}

#[derive(PartialEq, Debug)]
pub struct MilestoneSignature {
    signature: Vec<u8>,
}

impl MilestoneSignature {
    pub fn get_signature(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

impl core::fmt::Display for MilestoneSignature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.signature)
    }
}

#[derive(PartialEq, Debug)]
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
        self.essence
            .public_keys()
            .iter()
            .map(|key| key.try_into().unwrap())
            .collect()
    }

    pub fn receipt(&self) -> Option<ReceiptPayload> {
        let option = self.essence.receipt();
        if let Some(payload) = option {
            if let MessagePayloadRust::Receipt(receipt) = payload {
                return Some((*receipt.clone()).into());
            }
        }

        None
    }
}

impl core::fmt::Display for MilestonePayloadEssence {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.essence)
    }
}

pub struct PublicKey(RustPublicKey);

impl PublicKey {
    pub fn verify(&self, sig: Signature, msg: Vec<u8>) -> bool {
        self.0.verify(&sig.0, &msg)
    }

    pub fn to_compressed_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_compressed_bytes(bs: Vec<u8>) -> Result<Self> {
        let mut bs_arr: [u8; SECRET_KEY_LENGTH] = [0; SECRET_KEY_LENGTH];
        bs_arr.copy_from_slice(&bs[0..SECRET_KEY_LENGTH]);
        match RustPublicKey::try_from_bytes(bs_arr) {
            Ok(bytes) => Ok(Self(bytes)),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }
}
impl core::convert::TryFrom<&[u8; 32]> for PublicKey {
    type Error = anyhow::Error;
    fn try_from(bytes: &[u8; 32]) -> Result<Self, Self::Error> {
        match RustPublicKey::try_from_bytes(*bytes) {
            Ok(k) => Ok(Self(k)),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }
}

impl core::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            hex::encode(self.to_compressed_bytes())
        )
    }
}

pub struct Signature(RustSignature);

impl Signature {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_bytes(bs: Vec<u8>) -> Self {
        let mut bs_arr: [u8; SIGNATURE_LENGTH] = [0; SIGNATURE_LENGTH];
        bs_arr.copy_from_slice(&bs[0..SIGNATURE_LENGTH]);
        Self(RustSignature::from_bytes(bs_arr))
    }
}

impl core::fmt::Display for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            hex::encode(self.to_bytes())
        )
    }
}

impl From<RustSignature> for Signature {
    fn from(output: RustSignature) -> Self {
        Self(output)
    }
}
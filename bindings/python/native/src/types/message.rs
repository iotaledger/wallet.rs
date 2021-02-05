// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::{Error, Result};
// use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use core::convert::TryFrom;
use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota::{
    Address as RustAddress, Ed25519Address as RustEd25519Address, Ed25519Signature as RustEd25519Signature,
    IndexationPayload as RustIndexationPayload, Input as RustInput,
    MilestonePayloadEssence as RustMilestonePayloadEssence, Output as RustOutput, Payload as RustPayload,
    ReferenceUnlock as RustReferenceUnlock, SignatureLockedSingleOutput as RustSignatureLockedSingleOutput,
    SignatureUnlock as RustSignatureUnlock, TransactionId as RustTransationId,
    TransactionPayload as RustTransactionPayload, TransactionPayloadEssence as RustTransactionPayloadEssence,
    UTXOInput as RustUTXOInput, UnlockBlock as RustUnlockBlock,
};
// use iota::MessageId as RustMessageId,
use iota_wallet::message::Message as RustWalletMessage;
use std::{
    convert::{From, Into, TryInto},
    str::FromStr,
};

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;
// Note that we need a mechanism to update this constant as iota.rs, currently it is set as a
// constant for ease of python user to view the address with a constant prefix.
pub static mut BECH32_HRP: &str = "atoi1";

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
/// Message Type Wrapper for `Message` in wallet.rs
pub struct WalletMessage {
    /// The message identifier.
    pub id: String,
    /// The message version.
    pub version: u64,
    /// Message ids this message refers to.
    pub parents: Vec<String>,
    /// Length of the payload.
    pub payload_length: usize,
    /// Message payload.
    pub payload: Payload,
    /// The transaction timestamp.
    pub timestamp: i64,
    /// Transaction nonce.
    pub nonce: u64,
    /// Whether the transaction is confirmed or not.
    pub confirmed: Option<bool>,
    /// Whether the transaction is broadcasted or not.
    pub broadcasted: bool,
    /// Whether the message represents an incoming transaction or not.
    pub incoming: bool,
    /// The message's value.
    pub value: u64,
    /// The message's remainder value sum.
    pub remainder_value: u64,
}

impl TryFrom<RustWalletMessage> for WalletMessage {
    type Error = Error;
    fn try_from(msg: RustWalletMessage) -> Result<Self> {
        let payload = match msg.payload() {
            RustPayload::Transaction(payload) => Payload {
                transaction: Some(vec![Transaction {
                    essence: payload.essence().to_owned().try_into()?,
                    unlock_blocks: payload
                        .unlock_blocks()
                        .iter()
                        .cloned()
                        .map(|unlock_block| unlock_block.try_into().expect("Invalid UnlockBlock"))
                        .collect(),
                }]),
                milestone: None,
                indexation: None,
            },
            RustPayload::Indexation(payload) => Payload {
                transaction: None,
                milestone: None,
                indexation: Some(vec![Indexation {
                    index: payload.index().to_string(),
                    data: payload.data().try_into().unwrap_or_else(|_| {
                        panic!(
                            "invalid Indexation Payload {:?} with data: {:?}",
                            payload,
                            payload.data()
                        )
                    }),
                }]),
            },
            RustPayload::Milestone(payload) => Payload {
                transaction: None,
                milestone: Some(vec![Milestone {
                    essence: payload.essence().to_owned().try_into()?,
                    signatures: payload
                        .signatures()
                        .iter()
                        .map(|signature| (*signature).to_vec())
                        .collect(),
                }]),
                indexation: None,
            },
            _ => unreachable!(),
        };

        Ok(Self {
            id: msg.id().to_string(),
            version: *msg.version(),
            parents: msg.parents().iter().map(|parent| parent.to_string()).collect(),
            payload_length: *msg.payload_length(),
            payload,
            timestamp: msg.timestamp().timestamp(),
            nonce: *msg.nonce(),
            confirmed: *msg.confirmed(),
            broadcasted: *msg.broadcasted(),
            incoming: *msg.incoming(),
            value: *msg.value(),
            remainder_value: *msg.remainder_value(),
        })
    }
}

impl TryFrom<RustTransactionPayloadEssence> for TransactionPayloadEssence {
    type Error = Error;
    fn try_from(essence: RustTransactionPayloadEssence) -> Result<Self> {
        Ok(TransactionPayloadEssence {
            inputs: essence
                .inputs()
                .iter()
                .cloned()
                .map(|input| {
                    if let RustInput::UTXO(input) = input {
                        Input {
                            transaction_id: input.output_id().transaction_id().to_string(),
                            index: input.output_id().index(),
                        }
                    } else {
                        unreachable!()
                    }
                })
                .collect(),
            outputs: essence
                .outputs()
                .iter()
                .cloned()
                .map(|output| {
                    if let RustOutput::SignatureLockedSingle(output) = output {
                        Output {
                            address: unsafe { output.address().to_bech32(BECH32_HRP) },
                            amount: output.amount(),
                        }
                    } else {
                        unreachable!()
                    }
                })
                .collect(),
            payload: if essence.payload().is_some() {
                if let Some(RustPayload::Indexation(payload)) = essence.payload() {
                    Some(Payload {
                        transaction: None,
                        milestone: None,
                        indexation: Some(vec![Indexation {
                            index: payload.index().to_string(),
                            data: payload.data().try_into().unwrap_or_else(|_| {
                                panic!(
                                    "invalid Indexation Payload {:?} with data: {:?}",
                                    essence,
                                    payload.data()
                                )
                            }),
                        }]),
                    })
                } else {
                    unreachable!()
                }
            } else {
                None
            },
        })
    }
}

impl TryFrom<RustMilestonePayloadEssence> for MilestonePayloadEssence {
    type Error = Error;
    fn try_from(essence: RustMilestonePayloadEssence) -> Result<Self> {
        Ok(MilestonePayloadEssence {
            index: essence.index(),
            timestamp: essence.timestamp(),
            parents: essence.parents().iter().map(|parent| parent.to_string()).collect(),
            merkle_proof: essence.merkle_proof().try_into()?,
            public_keys: essence
                .public_keys()
                .iter()
                .map(|public_key| {
                    public_key.to_vec().try_into().unwrap_or_else(|_| {
                        panic!(
                            "invalid MilestonePayloadEssence {:?} with public key: {:?}",
                            essence,
                            essence.public_keys()
                        )
                    })
                })
                .collect(),
        })
    }
}

impl TryFrom<RustUnlockBlock> for UnlockBlock {
    type Error = Error;
    fn try_from(unlock_block: RustUnlockBlock) -> Result<Self> {
        if let RustUnlockBlock::Signature(RustSignatureUnlock::Ed25519(signature)) = unlock_block {
            Ok(UnlockBlock {
                signature: Some(Ed25519Signature {
                    public_key: signature.public_key().to_vec().try_into().unwrap_or_else(|_| {
                        panic!(
                            "invalid Ed25519Signature {:?} with public key: {:?}",
                            signature,
                            signature.public_key()
                        )
                    }),
                    signature: signature.signature().to_vec(),
                }),
                reference: None,
            })
        } else if let RustUnlockBlock::Reference(signature) = unlock_block {
            Ok(UnlockBlock {
                signature: None,
                reference: Some(signature.index()),
            })
        } else {
            unreachable!()
        }
    }
}

// Note: This conversion be binded only if all fields of the `RustWalletMessage` can be set publically.
impl TryFrom<WalletMessage> for RustWalletMessage {
    type Error = Error;
    fn try_from(_msg: WalletMessage) -> Result<Self> {
        // Ok(Self {
        //     id: RustMessageId::from_str(&msg.id)?,
        //     version: msg.version,
        //     parent1: RustMessageId::from_str(&msg.parent1)?,
        //     parent2: RustMessageId::from_str(&msg.parent2)?,
        //     payload_length: msg.payload_length,
        //     payload: msg.payload.try_into()?,
        //     timestamp: DateTime::from_utc(NaiveDateTime::from_timestamp(msg.timestamp, 0), Utc),
        //     nonce: msg.nonce,
        //     confirmed: msg.confirmed,
        //     broadcasted: msg.broadcasted,
        //     incoming: msg.incoming,
        //     value: msg.value,
        //     remainder_value: msg.remainder_value,
        // })
        todo!();
    }
}

impl TryFrom<TransactionPayloadEssence> for RustTransactionPayloadEssence {
    type Error = Error;
    fn try_from(essence: TransactionPayloadEssence) -> Result<Self> {
        let mut builder = RustTransactionPayloadEssence::builder();
        let inputs: Vec<RustInput> = essence
            .inputs
            .iter()
            .map(|input| {
                RustUTXOInput::new(
                    RustTransationId::from_str(&input.transaction_id[..]).unwrap_or_else(|_| {
                        panic!(
                            "invalid UTXOInput transaction_id: {} with input index {}",
                            input.transaction_id, input.index
                        )
                    }),
                    input.index,
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "invalid UTXOInput transaction_id: {} with input index {}",
                        input.transaction_id, input.index
                    )
                })
                .into()
            })
            .collect();
        for input in inputs {
            builder = builder.add_input(input);
        }

        let outputs: Vec<RustOutput> = essence
            .outputs
            .iter()
            .map(|output| {
                RustSignatureLockedSingleOutput::new(
                    RustAddress::from(RustEd25519Address::from_str(&output.address[..]).unwrap_or_else(|_| {
                        panic!(
                            "invalid SignatureLockedSingleOutput with output address: {}",
                            output.address
                        )
                    })),
                    output.amount,
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "invalid SignatureLockedSingleOutput with output address: {}",
                        output.address
                    )
                })
                .into()
            })
            .collect();
        for output in outputs {
            builder = builder.add_output(output);
        }
        if let Some(indexation_payload) = &essence.payload {
            let index = RustIndexationPayload::new(
                indexation_payload
                    .indexation
                    .as_ref()
                    .unwrap_or_else(|| panic!("Invalid IndexationPayload: {:?}", indexation_payload))[0]
                    .index
                    .clone(),
                &(indexation_payload
                    .indexation
                    .as_ref()
                    .unwrap_or_else(|| panic!("Invalid IndexationPayload: {:?}", indexation_payload))[0]
                    .data)
                    .clone(),
            )
            .unwrap();
            builder = builder.with_payload(RustPayload::from(index));
        }
        Ok(builder.finish()?)
    }
}

impl TryFrom<Ed25519Signature> for RustSignatureUnlock {
    type Error = Error;
    fn try_from(signature: Ed25519Signature) -> Result<Self> {
        let mut public_key = [0u8; 32];
        hex::decode_to_slice(signature.public_key, &mut public_key)?;
        let signature = hex::decode(signature.signature)?.into_boxed_slice();
        Ok(RustEd25519Signature::new(public_key, signature).into())
    }
}

impl TryFrom<UnlockBlock> for RustUnlockBlock {
    type Error = Error;
    fn try_from(block: UnlockBlock) -> Result<Self> {
        if let Some(signature) = block.signature {
            let sig: RustSignatureUnlock = signature.try_into()?;
            Ok(sig.into())
        } else {
            let reference: RustReferenceUnlock = block
                .reference
                .unwrap()
                .try_into()
                .unwrap_or_else(|_| panic!("Invalid ReferenceUnlock: {:?}", block.reference));
            Ok(reference.into())
        }
    }
}

impl TryFrom<Payload> for RustPayload {
    type Error = Error;
    fn try_from(payload: Payload) -> Result<Self> {
        if let Some(transaction_payload) = &payload.transaction {
            let mut transaction = RustTransactionPayload::builder();
            transaction = transaction.with_essence(transaction_payload[0].essence.clone().try_into()?);

            let unlock_blocks = transaction_payload[0].unlock_blocks.clone();
            for unlock_block in unlock_blocks {
                transaction = transaction.add_unlock_block(unlock_block.try_into()?);
            }

            Ok(RustPayload::Transaction(Box::new(transaction.finish()?)))
        } else {
            let indexation = RustIndexationPayload::new(
                (&payload
                    .indexation
                    .as_ref()
                    .unwrap_or_else(|| panic!("Invalid Payload: {:?}", payload))[0]
                    .index
                    .clone())
                    .to_owned(),
                &payload
                    .indexation
                    .as_ref()
                    .unwrap_or_else(|| panic!("Invalid Payload: {:?}", payload))[0]
                    .data,
            )?;
            Ok(RustPayload::Indexation(Box::new(indexation)))
        }
    }
}

impl TryFrom<Indexation> for RustIndexationPayload {
    type Error = Error;
    fn try_from(indexation: Indexation) -> Result<Self> {
        Ok(RustIndexationPayload::new(indexation.index, &indexation.data)?)
    }
}

/// Message Type Wrapper for `IotaMessage`
#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Message {
    pub message_id: String,
    pub network_id: u64,
    pub parents: Vec<String>,
    pub payload: Option<Payload>,
    pub nonce: u64,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Payload {
    pub transaction: Option<Vec<Transaction>>,
    pub milestone: Option<Vec<Milestone>>,
    pub indexation: Option<Vec<Indexation>>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Transaction {
    pub essence: TransactionPayloadEssence,
    pub unlock_blocks: Vec<UnlockBlock>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Milestone {
    pub essence: MilestonePayloadEssence,
    pub signatures: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct MilestonePayloadEssence {
    pub index: u32,
    pub timestamp: u64,
    pub parents: Vec<String>,
    pub merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    pub public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Indexation {
    pub index: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct TransactionPayloadEssence {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub payload: Option<Payload>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Output {
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Input {
    pub transaction_id: String,
    pub index: u16,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct UnlockBlock {
    pub signature: Option<Ed25519Signature>,
    pub reference: Option<u16>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Ed25519Signature {
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

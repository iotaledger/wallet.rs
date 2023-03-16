// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    error::{Error, Result},
    AddressOutput,
};
use chrono::prelude::{DateTime, NaiveDateTime, Utc};
use core::convert::TryFrom;
use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_client::bee_message::prelude::{
    Address as RustAddress, Ed25519Address as RustEd25519Address, Ed25519Signature as RustEd25519Signature,
    Essence as RustEssence, IndexationPayload as RustIndexationPayload, Input as RustInput, Output as RustOutput,
    Payload as RustPayload, ReferenceUnlock as RustReferenceUnlock, RegularEssence as RustRegularEssence,
    SignatureLockedSingleOutput as RustSignatureLockedSingleOutput, SignatureUnlock as RustSignatureUnlock,
    TransactionId as RustTransationId, TransactionPayload as RustTransactionPayload, UnlockBlock as RustUnlockBlock,
    UnlockBlocks as RustUnlockBlocks, UtxoInput as RustUtxoInput,
};
// use iota_client::bee_message::MessageId as RustMessageId,
use iota_client::bee_message::prelude::{Address as IotaAddress, MessageId, TransactionId};
use iota_wallet::{
    account_manager::AccountStore,
    address::{
        Address as RustWalletAddress, AddressOutput as RustWalletAddressOutput, AddressWrapper as RustAddressWrapper,
        OutputKind as RustOutputKind,
    },
    client::ClientOptions as RustWalletClientOptions,
    message::{
        Message as RustWalletMessage, MessageMilestonePayloadEssence as RustWalletMilestonePayloadEssence,
        MessagePayload as RustWalletPayload, MessageTransactionPayload as RustWalletMessageTransactionPayload,
        TransactionBuilderMetadata as RustWalletTransactionBuilderMetadata,
        TransactionEssence as RustWalletTransactionEssence, TransactionInput as RustWalletInput,
        TransactionOutput as RustWalletOutput,
    },
};
use std::{
    convert::{From, Into, TryInto},
    str::FromStr,
};

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

type Bech32Hrp = String;

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct WalletAddressOutput {
    pub transaction_id: String,
    pub message_id: String,
    pub index: u16,
    pub amount: u64,
    pub is_spent: bool,
    pub address: String,
    pub kind: String,
}

impl TryFrom<WalletAddressOutput> for RustWalletAddressOutput {
    type Error = Error;
    fn try_from(output: WalletAddressOutput) -> Result<Self> {
        Ok(RustWalletAddressOutput {
            transaction_id: TransactionId::new(hex::decode(output.transaction_id)?[..].try_into()?),
            message_id: MessageId::new(hex::decode(output.message_id)?[..].try_into()?),
            index: output.index,
            amount: output.amount,
            is_spent: output.is_spent,
            // we use an empty bech32 HRP here because we update it later on wallet.rs
            address: RustAddressWrapper::new(IotaAddress::try_from_bech32(&output.address)?, "".to_string()),
            kind: RustOutputKind::from_str(&output.kind)?,
        })
    }
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct WalletAddress {
    pub address: String,
    pub key_index: usize,
    pub internal: bool,
    pub outputs: Vec<WalletAddressOutput>,
}

impl TryFrom<WalletAddress> for RustWalletAddress {
    type Error = Error;
    fn try_from(address: WalletAddress) -> Result<Self> {
        let mut outputs = Vec::new();
        for output in address.outputs {
            outputs.push(output.try_into()?);
        }
        let address = RustWalletAddress::builder()
            // we use an empty bech32 HRP here because we update it later on wallet.rs
            .address(RustAddressWrapper::new(
                IotaAddress::try_from_bech32(&address.address)?,
                "".to_string(),
            ))
            .key_index(address.key_index)
            .internal(address.internal)
            .outputs(outputs)
            .build()?;
        Ok(address)
    }
}

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
    pub payload: Option<Payload>,
    /// The transaction timestamp.
    pub timestamp: i64,
    /// Transaction nonce.
    pub nonce: u64,
    /// Whether the transaction is confirmed or not.
    pub confirmed: Option<bool>,
    /// Whether the transaction is broadcasted or not.
    pub broadcasted: bool,
}

impl TryFrom<RustWalletMessage> for WalletMessage {
    type Error = Error;
    fn try_from(msg: RustWalletMessage) -> Result<Self> {
        let payload = match msg.payload() {
            Some(RustWalletPayload::Transaction(payload)) => Some(Payload {
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
            }),
            Some(RustWalletPayload::Indexation(payload)) => Some(Payload {
                transaction: None,
                milestone: None,
                indexation: Some(vec![Indexation {
                    index: payload.index().to_vec(),
                    data: payload.data().try_into().unwrap_or_else(|_| {
                        panic!(
                            "invalid Indexation Payload {:?} with data: {:?}",
                            payload,
                            payload.data()
                        )
                    }),
                }]),
            }),
            Some(RustWalletPayload::Milestone(payload)) => Some(Payload {
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
            }),
            Some(_) => unimplemented!(),
            None => None,
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
        })
    }
}

impl TryFrom<RustWalletTransactionEssence> for Essence {
    type Error = Error;
    fn try_from(essence: RustWalletTransactionEssence) -> Result<Self> {
        let essence = match essence {
            RustWalletTransactionEssence::Regular(essence) => RegularEssence {
                inputs: essence
                    .inputs()
                    .iter()
                    .cloned()
                    .map(|input| {
                        if let RustWalletInput::Utxo(input) = input {
                            Input {
                                transaction_id: input.input.output_id().transaction_id().to_string(),
                                index: input.input.output_id().index(),
                                metadata: input.metadata.map(|m| (&m).into()),
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
                        if let RustWalletOutput::SignatureLockedSingle(output) = output {
                            Output {
                                address: output.address().to_bech32(),
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
                                index: payload.index().to_vec(),
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
                internal: essence.internal(),
                incoming: essence.incoming(),
                value: essence.value(),
                remainder_value: essence.remainder_value(),
            }
            .into(),
        };
        Ok(essence)
    }
}

impl TryFrom<RustWalletMilestonePayloadEssence> for MilestonePayloadEssence {
    type Error = Error;
    fn try_from(essence: RustWalletMilestonePayloadEssence) -> Result<Self> {
        Ok(MilestonePayloadEssence {
            index: *essence.index(),
            timestamp: essence.timestamp(),
            parents: essence.parents().iter().map(|parent| parent.to_string()).collect(),
            merkle_proof: *essence.merkle_proof(),
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

pub async fn to_rust_message(
    msg: WalletMessage,
    bech32_hrp: String,
    accounts: AccountStore,
    account_id: &str,
    account_addresses: &[RustWalletAddress],
    client_options: &RustWalletClientOptions,
) -> Result<RustWalletMessage> {
    let mut parents = Vec::new();
    for parent in msg.parents {
        parents.push(MessageId::from_str(&parent)?);
    }
    let id = MessageId::from_str(&msg.id)?;
    let payload = match msg.payload {
        Some(payload) => Some(
            to_rust_payload(
                &id,
                payload,
                bech32_hrp,
                accounts,
                account_id,
                account_addresses,
                client_options,
            )
            .await?,
        ),
        None => None,
    };
    Ok(RustWalletMessage {
        id,
        version: msg.version,
        parents,
        payload_length: msg.payload_length,
        payload,
        timestamp: DateTime::from_utc(NaiveDateTime::from_timestamp(msg.timestamp, 0), Utc),
        nonce: msg.nonce,
        confirmed: msg.confirmed,
        broadcasted: msg.broadcasted,
        reattachment_message_id: None,
    })
}

impl TryFrom<Essence> for RustEssence {
    type Error = Error;

    fn try_from(essence: Essence) -> Result<Self> {
        if let Some(essence) = essence.regular {
            let mut builder = RustRegularEssence::builder();
            let inputs: Vec<RustInput> = essence
                .inputs
                .iter()
                .map(|input| {
                    RustUtxoInput::new(
                        RustTransationId::from_str(&input.transaction_id[..]).unwrap_or_else(|_| {
                            panic!(
                                "invalid UtxoInput transaction_id: {} with input index {}",
                                input.transaction_id, input.index
                            )
                        }),
                        input.index,
                    )
                    .unwrap_or_else(|_| {
                        panic!(
                            "invalid UtxoInput transaction_id: {} with input index {}",
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
                    &indexation_payload
                        .indexation
                        .as_ref()
                        .unwrap_or_else(|| panic!("Invalid IndexationPayload: {indexation_payload:?}"))[0]
                        .index
                        .clone(),
                    &(indexation_payload
                        .indexation
                        .as_ref()
                        .unwrap_or_else(|| panic!("Invalid IndexationPayload: {indexation_payload:?}"))[0]
                        .data)
                        .clone(),
                )
                .unwrap();
                builder = builder.with_payload(RustPayload::from(index));
            }
            Ok(RustEssence::Regular(builder.finish()?))
        } else {
            unimplemented!()
        }
    }
}

impl TryFrom<Ed25519Signature> for RustSignatureUnlock {
    type Error = Error;
    fn try_from(signature: Ed25519Signature) -> Result<Self> {
        let mut public_key = [0u8; 32];
        hex::decode_to_slice(signature.public_key, &mut public_key)?;
        let signature = hex::decode(signature.signature)?[..].try_into()?;
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

pub async fn to_rust_payload(
    message_id: &MessageId,
    payload: Payload,
    bech32_hrp: Bech32Hrp,
    accounts: AccountStore,
    account_id: &str,
    account_addresses: &[RustWalletAddress],
    client_options: &RustWalletClientOptions,
) -> Result<RustWalletPayload> {
    if let Some(transaction_payload) = &payload.transaction {
        let mut transaction = RustTransactionPayload::builder();
        transaction = transaction.with_essence(transaction_payload[0].essence.clone().try_into()?);

        let unlock_blocks: Result<Vec<RustUnlockBlock>> = transaction_payload[0]
            .unlock_blocks
            .iter()
            .cloned()
            .map(|u| u.try_into())
            .collect();
        transaction = transaction.with_unlock_blocks(RustUnlockBlocks::new(unlock_blocks?)?);
        let metadata = RustWalletTransactionBuilderMetadata {
            id: message_id,
            bech32_hrp,
            account_id,
            accounts,
            account_addresses,
            client_options,
        };
        Ok(RustWalletPayload::Transaction(Box::new(
            RustWalletMessageTransactionPayload::new(&transaction.finish()?, &metadata).await?,
        )))
    } else {
        let indexation = RustIndexationPayload::new(
            &payload
                .indexation
                .as_ref()
                .unwrap_or_else(|| panic!("Invalid Payload: {payload:?}"))[0]
                .index
                .clone(),
            &payload
                .indexation
                .as_ref()
                .unwrap_or_else(|| panic!("Invalid Payload: {payload:?}"))[0]
                .data,
        )?;
        Ok(RustWalletPayload::Indexation(Box::new(indexation)))
    }
}

impl TryFrom<Indexation> for RustIndexationPayload {
    type Error = Error;
    fn try_from(indexation: Indexation) -> Result<Self> {
        Ok(RustIndexationPayload::new(&indexation.index, &indexation.data)?)
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
    pub essence: Essence,
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
    pub index: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct Essence {
    regular: Option<RegularEssence>,
}

#[derive(Debug, Clone, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct RegularEssence {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub payload: Option<Payload>,
    pub internal: bool,
    /// Whether the transaction is incoming (received) or outgoing (sent).
    pub incoming: bool,
    /// The transaction's value.
    pub value: u64,
    /// The transaction's remainder value sum.
    pub remainder_value: u64,
}

impl From<RegularEssence> for Essence {
    fn from(essence: RegularEssence) -> Self {
        Self { regular: Some(essence) }
    }
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
    pub metadata: Option<AddressOutput>,
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

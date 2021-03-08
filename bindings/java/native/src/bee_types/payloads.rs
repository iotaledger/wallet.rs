use iota_wallet::{
    message::{
        MessagePayload as MessagePayloadRust,
        ReceiptPayload as ReceiptPayloadRust,
        TreasuryTransactionPayload as TreasuryTransactionPayloadRust,
        MessageId,
    },
};

use iota::{
    Address as RustAddress, Ed25519Address as RustEd25519Address, Ed25519Signature as RustEd25519Signature,
    Essence as RustEssence, IndexationPayload as RustIndexationPayload, Input as RustInput,
    Output as RustOutput, Payload as RustPayload,
    ReferenceUnlock as RustReferenceUnlock, RegularEssence as RustRegularEssence,
    SignatureLockedSingleOutput as RustSignatureLockedSingleOutput, SignatureUnlock as RustSignatureUnlock,
    TransactionId as RustTransationId, TransactionPayload as RustTransactionPayload, UTXOInput as RustUTXOInput,
    UnlockBlock as RustUnlockBlock, UnlockBlocks as RustUnlockBlocks,
};

use crate::bee_types::index::*;
use crate::bee_types::milestone::*;
use crate::bee_types::transaction::*;

// TreasuryInput, TreasuryOutput
// Essence, UTXOInput, UnlockBlock
// RegularEssence, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput

use crate::Result;

use anyhow::anyhow;

pub enum MessagePayloadType {
    Transaction = 1,
    Milestone = 2,
    Indexation = 3,
    Receipt = 4,
    TreasuryTransaction = 5,
}

pub struct MessagePayload {
    payload: MessagePayloadRust,
}

impl MessagePayload {
    pub fn get_internal(self) -> MessagePayloadRust {
        self.payload
    }

    pub fn new_with_internal(payload: MessagePayloadRust) -> Self {
        MessagePayload {
            payload: payload,
        }
    } 

    pub fn payload_type(&self) -> MessagePayloadType {
        match self.payload {
            MessagePayloadRust::Transaction(_) => MessagePayloadType::Transaction,
            MessagePayloadRust::Milestone(_) => MessagePayloadType::Milestone,
            MessagePayloadRust::Indexation(_) => MessagePayloadType::Indexation,
            MessagePayloadRust::Receipt(_) => MessagePayloadType::Receipt,
            MessagePayloadRust::TreasuryTransaction(_) => MessagePayloadType::TreasuryTransaction,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.payload)
    }

    pub fn get_as_transaction(&self) -> Option<MessageTransactionPayload> {
        if let MessagePayloadRust::Transaction(payload) = &self.payload {
            Some(MessageTransactionPayload::new_with_rust(payload))
        } else {
            None
        }
    }

    pub fn get_as_indexation(&self) -> Option<IndexationPayload> {
        if let MessagePayloadRust::Indexation(index) = &self.payload {
            match IndexationPayload::new_with(index.index(), index.data()) {
                Ok(i) => Some(i),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_as_milestone(&self) -> Option<MilestonePayload> {
        if let MessagePayloadRust::Milestone(payload) = &self.payload {
            match MilestonePayload::new_with(
                payload.essence().to_owned(),
                payload.signatures().to_owned()
            ) {
                Ok(i) => Some(i),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
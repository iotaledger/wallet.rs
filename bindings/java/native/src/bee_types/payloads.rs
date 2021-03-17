use iota_wallet::{
    message::{
        MessagePayload as MessagePayloadRust,
    },
};

use crate::bee_types::index::*;
use crate::bee_types::milestone::*;
use crate::bee_types::transaction::*;
use crate::bee_types::receipt::*;
use crate::bee_types::treasury::*;

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

    pub fn get_as_receipt(&self) -> Option<ReceiptPayload> {
        if let MessagePayloadRust::Receipt(payload) = &self.payload {
            Some(ReceiptPayload::new_with_rust(*payload.clone()))
        } else {
            None
        }
    }

    pub fn get_as_treasury(&self) -> Option<TreasuryTransactionPayload> {
        if let MessagePayloadRust::TreasuryTransaction(payload) = &self.payload {
            Some(TreasuryTransactionPayload::new_with_rust(*payload.clone()))
        } else {
            None
        }
    }
}

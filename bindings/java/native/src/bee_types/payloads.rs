use iota_wallet::{
    message::{
        IndexationPayload as IndexationPayloadRust,
        MessagePayload as MessagePayloadRust,
    },
};

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
}

pub struct IndexationPayload {
    payload: IndexationPayloadRust,
}

impl IndexationPayload {
    pub fn get_internal(self) -> IndexationPayloadRust {
        self.payload
    }

    pub fn new_with(index: &[u8], data: &[u8]) -> Result<IndexationPayload> {
        let index = IndexationPayloadRust::new(&index, &data);
        match index {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(i) => Ok(IndexationPayload {
                payload: i
            })
        }
    }

    pub fn index(&self) -> &[u8] {
        self.payload.index()
    }

    pub fn data(&self) -> &[u8] {
        self.payload.data()
    }
}



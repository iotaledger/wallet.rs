use std::{
    cell::RefCell,
    rc::Rc,
};

use iota_wallet::{
    message::{
        IndexationPayload as IndexationPayloadRust,
        MessagePayload as MessagePayloadRust,
    },
};

use crate::Result;

use anyhow::anyhow;

pub struct IndexationPayload {
    payload: IndexationPayloadRust,
}

impl IndexationPayload {
    pub fn get_internal(self) -> IndexationPayloadRust {
        // TODO: Find a way to not need clone
        self.payload
    }

    pub fn new(index: &[u8], data: &[u8]) -> Result<IndexationPayload> {
        let index = IndexationPayloadRust::new(index, data);
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

pub struct MessagePayload {
    payload: MessagePayloadRust,
}

impl MessagePayload {
    pub fn new_with_internal(payload: MessagePayloadRust) -> Self {
        MessagePayload {
            payload: payload,
        }
    } 
}

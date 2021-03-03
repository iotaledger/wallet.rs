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

pub struct IndexationPayload {
    payload: IndexationPayloadRust,
}

impl IndexationPayload {
    pub fn get_internal(self) -> IndexationPayloadRust {
        // TODO: Find a way to not need clone
        self.payload
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

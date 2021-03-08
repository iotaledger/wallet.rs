use anyhow::anyhow;
use crate::Result;

use iota_wallet::{
    message::{
        IndexationPayload as IndexationPayloadRust,
    }
};

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
use iota_wallet::{
    message::{
        ReceiptPayload as ReceiptPayloadRust,
    }
};

use iota::{
    MigratedFundsEntry as MigratedFundsEntryRust,
    SignatureLockedSingleOutput,
};

pub struct ReceiptPayload {
    payload: ReceiptPayloadRust,
}

impl ReceiptPayload {

    pub fn new_with_rust(payload: ReceiptPayloadRust) -> Self {
        Self {
            payload: payload
        }
    }

    pub fn index(&self) -> u32 {
        self.payload.index()
    }

    pub fn last(&self) -> bool {
        self.payload.last()
    }

    
    pub fn funds(&self) -> Vec<MigratedFundsEntry> {
        self.payload.funds().into_iter().map(|m| MigratedFundsEntry { payload: m.clone()}).collect()
    }
    /*
    pub fn transaction(&self) -> Payload {
        self.payload.transaction().clone()
    }*/

    pub fn amount(&self) -> u64 {
        self.payload.amount()
    }
}

pub struct MigratedFundsEntry {
    payload: MigratedFundsEntryRust,
}

impl MigratedFundsEntry {
    pub fn tail_transaction_hash(&self) -> Vec<u8> {
        self.payload.tail_transaction_hash().into_iter().map(|h| *h).collect()
    }

    pub fn output(&self) -> SignatureLockedSingleOutput {
        self.payload.output().clone()
    }
}


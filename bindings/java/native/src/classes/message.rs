use iota_wallet::{
    message::{
        Message as MessageRust,
        MessageId, Payload,
    },
};

use chrono::prelude::{DateTime, Utc};

pub struct Message {
    message: MessageRust
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            message: self.message.clone()
        }
    }
}

impl Message {
    pub fn id(&self) -> MessageId{
        self.message.id().clone()
    }
    pub fn version(&self) -> u64 {
        *(self.message.version())
    }
    pub fn parents(&self) -> Vec<MessageId> {
        self.message.parents().to_vec()
    }
    pub fn payload_length(&self) -> usize {
        *(self.message.payload_length())
    }
    pub fn payload(&self) -> Payload {
        self.message.payload().clone()
    }
    pub fn timestamp(&self) -> DateTime<Utc> {
        *(self.message.timestamp())
    }
    pub fn nonce(&self) -> u64{
        *(self.message.nonce())
    }
    pub fn confirmed(&self) -> Option<bool> {
        *(self.message.confirmed())
    }
    pub fn broadcasted(&self) -> bool {
        *(self.message.broadcasted())
    }
    pub fn incoming(&self) -> bool {
        *(self.message.incoming())
    }
    pub fn value(&self) -> u64 {
        *(self.message.value())
    }
    pub fn remainder_value(&self) -> u64 {
        *(self.message.remainder_value())
    }

    pub fn get_internal(self) -> MessageRust {
        // TODO: Find a way to not need clone
        self.message
 
    }
}
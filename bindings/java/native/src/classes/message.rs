// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use iota_wallet::{
    address::{AddressWrapper, OutputKind},
    message::{
        Message as MessageRust, BlockId, RemainderValueStrategy as RemainderValueStrategyRust,
        Transaction as TransactionRust, TransactionBuilder as TransactionBuilderRust,
    },
};

use crate::types::{IndexationPayload, MessagePayload};

use chrono::prelude::{DateTime, Utc};
use std::num::NonZeroU64;

pub enum RemainderValueStrategy {
    ReuseAddress = 1,
    ChangeAddress = 2,
}

pub fn remainder_type_enum_to_type(strategy: RemainderValueStrategy) -> RemainderValueStrategyRust {
    match strategy {
        RemainderValueStrategy::ReuseAddress => RemainderValueStrategyRust::ReuseAddress,
        RemainderValueStrategy::ChangeAddress => RemainderValueStrategyRust::ChangeAddress,
    }
}

pub struct Transaction {
    transaction: TransactionRust,
}

impl Transaction {
    pub fn to_inner(self) -> TransactionRust {
        self.transaction
    }

    pub fn builder(address: AddressWrapper, amount: u64, output_kind: Option<OutputKind>) -> TransactionBuilder {
        TransactionBuilder::new(address, amount, output_kind)
    }
}

pub struct TransactionBuilder {
    builder: Rc<RefCell<Option<TransactionBuilderRust>>>,
}

impl TransactionBuilder {
    pub fn new(address: AddressWrapper, amount: u64, output_kind: Option<OutputKind>) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(TransactionBuilderRust::new(
                address,
                NonZeroU64::new(amount).unwrap(),
                output_kind,
            )))),
        }
    }

    pub fn new_with_builder(builder: TransactionBuilderRust) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder))),
        }
    }

    pub fn with_remainder_value_strategy(&mut self, strategy: RemainderValueStrategy) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_remainder_value_strategy(remainder_type_enum_to_type(strategy));
        TransactionBuilder::new_with_builder(new_builder)
    }

    pub fn with_remainder_to_account_with_address(&mut self, address: AddressWrapper) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_remainder_value_strategy(RemainderValueStrategyRust::AccountAddress(address));
        TransactionBuilder::new_with_builder(new_builder)
    }

    pub fn with_indexation(&mut self, indexation: IndexationPayload) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_indexation(indexation.to_inner());
        TransactionBuilder::new_with_builder(new_builder)
    }

    pub fn with_skip_sync(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_skip_sync();
        TransactionBuilder::new_with_builder(new_builder)
    }

    /// Builds the transaction.
    pub fn finish(&self) -> Transaction {
        Transaction {
            transaction: self.builder.borrow_mut().take().unwrap().finish(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Message {
    message: MessageRust,
}

impl From<MessageRust> for Message {
    fn from(message: MessageRust) -> Self {
        Self { message }
    }
}

impl Message {
    pub fn id(&self) -> BlockId {
        self.message.id().clone()
    }
    pub fn version(&self) -> u64 {
        *(self.message.version())
    }
    pub fn parents(&self) -> Vec<BlockId> {
        self.message.parents().to_vec()
    }
    pub fn payload_length(&self) -> usize {
        *(self.message.payload_length())
    }

    pub fn payload(&self) -> Option<MessagePayload> {
        match self.message.payload() {
            None => None,
            Some(e) => Some(e.clone().into()),
        }
    }
    pub fn timestamp(&self) -> DateTime<Utc> {
        *(self.message.timestamp())
    }
    pub fn nonce(&self) -> u64 {
        *(self.message.nonce())
    }
    pub fn confirmed(&self) -> Option<bool> {
        *(self.message.confirmed())
    }
    pub fn broadcasted(&self) -> bool {
        *(self.message.broadcasted())
    }

    pub fn to_inner(self) -> MessageRust {
        self.message
    }
}

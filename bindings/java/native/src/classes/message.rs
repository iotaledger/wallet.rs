// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use iota_wallet::{
    address::AddressWrapper,
    message::{
        Message as MessageRust, MessageId, RemainderValueStrategy as RemainderValueStrategyRust,
        Transfer as TransferRust, TransferBuilder as TransferBuilderRust, TransferOutput as TransferOutputRust,
    },
};

use crate::{
    types::{output_kind_enum_to_type, IndexationPayload, MessagePayload, OutputKind},
    Result,
};

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

#[derive(Debug, Clone)]
pub struct Transfer(TransferRust);

impl Transfer {
    pub fn to_inner(self) -> TransferRust {
        self.0
    }

    pub fn builder(address: AddressWrapper, amount: u64, output_kind: OutputKind) -> TransferBuilder {
        TransferBuilder::new(address, amount, output_kind)
    }
}

impl core::fmt::Display for Transfer {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct TransferOutput(TransferOutputRust);

impl TransferOutput {
    pub fn new(address: AddressWrapper, amount: u64, output_kind: OutputKind) -> TransferOutput {
        Self(TransferOutputRust::new(
            address,
            NonZeroU64::new(amount).unwrap(),
            output_kind_enum_to_type(output_kind),
        ))
    }

    pub fn get_amount(&mut self) -> u64 {
        self.0.amount.into()
    }

    pub fn get_address(&mut self) -> AddressWrapper {
        self.0.address.clone()
    }

    pub fn get_output_kind(&mut self) -> OutputKind {
        self.0.output_kind.clone().into()
    }

    pub fn to_inner(self) -> TransferOutputRust {
        self.0
    }
}

impl core::fmt::Display for TransferOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

pub struct TransferBuilder {
    builder: Rc<RefCell<Option<TransferBuilderRust>>>,
}

impl TransferBuilder {
    pub fn new(address: AddressWrapper, amount: u64, output_kind: OutputKind) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(TransferBuilderRust::new(
                address,
                NonZeroU64::new(amount).unwrap(),
                output_kind_enum_to_type(output_kind),
            )))),
        }
    }

    pub fn new_from_outputs(outputs: Vec<TransferOutput>) -> Result<Self> {
        match TransferBuilderRust::with_outputs(outputs.iter().map(|o| o.clone().0).collect()) {
            Ok(b) => Ok(TransferBuilder::new_with_builder(b)),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }

    pub fn new_with_builder(builder: TransferBuilderRust) -> Self {
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
        TransferBuilder::new_with_builder(new_builder)
    }

    pub fn with_remainder_to_account_with_address(&mut self, address: AddressWrapper) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_remainder_value_strategy(RemainderValueStrategyRust::AccountAddress(address));
        TransferBuilder::new_with_builder(new_builder)
    }

    pub fn with_indexation(&mut self, indexation: IndexationPayload) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_indexation(indexation.to_inner());
        TransferBuilder::new_with_builder(new_builder)
    }

    pub fn with_skip_sync(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_skip_sync();
        TransferBuilder::new_with_builder(new_builder)
    }

    /// Builds the transfer.
    pub fn finish(&self) -> Transfer {
        Transfer(self.builder.borrow_mut().take().unwrap().finish())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    message: MessageRust,
}

impl From<MessageRust> for Message {
    fn from(message: MessageRust) -> Self {
        Self { message }
    }
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

impl Message {
    pub fn id(&self) -> MessageId {
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

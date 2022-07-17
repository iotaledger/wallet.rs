// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use getset::{CopyGetters, Getters, Setters};
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use iota_wallet::{
    address::{
        Address as AddressRust, AddressBuilder as AddressBuilderRust, AddressOutput as AddressOutputRust,
        AddressWrapper, OutputKind,
    },
    iota_client::block::{payload::transaction::TransactionId, prelude::OutputId},
    message::BlockId,
};

use crate::Result;

#[derive(Clone, PartialEq, Debug)]
pub struct Address(AddressRust);

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({})", self.readable())
    }
}

impl From<AddressRust> for Address {
    fn from(address: AddressRust) -> Self {
        Self(address)
    }
}

impl From<&AddressRust> for Address {
    fn from(address: &AddressRust) -> Self {
        Self(address.clone())
    }
}

impl Address {
    pub fn builder() -> AddressBuilder {
        AddressBuilder::new()
    }

    pub fn readable(&self) -> String {
        self.0.address().to_bech32()
    }

    pub fn balance(&self) -> u64 {
        self.0.balance()
    }

    pub fn to_inner(self) -> AddressRust {
        // TODO: Find a way to not need clone
        self.0.clone()
    }

    pub fn address(&self) -> AddressWrapper {
        self.0.address().clone()
    }
}

pub struct AddressBuilder {
    builder: Rc<RefCell<Option<AddressBuilderRust>>>,
}

impl AddressBuilder {
    /// Initialises a new instance of the address builder.
    pub fn new() -> Self {
        AddressBuilder::new_with_builder(AddressBuilderRust::new())
    }

    fn new_with_builder(builder: AddressBuilderRust) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder))),
        }
    }

    pub fn address(&self, address: AddressWrapper) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().address(address);
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn key_index(&self, key_index: usize) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().key_index(key_index);
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn outputs(&self, outputs: Vec<AddressOutput>) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .outputs(outputs.iter().map(|o| o.clone().to_inner()).collect());
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn internal(&self, internal: bool) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().internal(internal);
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn build(&self) -> Result<Address> {
        match self.builder.borrow_mut().take().unwrap().build() {
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
            Ok(address) => Ok(Address(address)),
        }
    }
}

/// An Address output.
#[derive(Debug, Getters, CopyGetters, Setters, Clone, PartialEq)]
pub struct AddressOutput {
    /// Transaction ID of the output
    #[getset(get_copy = "pub")]
    pub transaction_id: TransactionId,
    /// Message ID of the output
    #[getset(get_copy = "pub")]
    pub block_id: BlockId,
    /// Output index.
    #[getset(get_copy = "pub")]
    pub index: u16,
    /// Output amount.
    #[getset(get_copy = "pub")]
    pub amount: u64,
    /// Spend status of the output,
    #[getset(get_copy = "pub")]
    pub is_spent: bool,
    /// Associated address.
    pub address: AddressWrapper,
    /// Output kind.
    pub kind: OutputKind,
}

impl Display for AddressOutput {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "(transaction_id={}, block_id={}, index={}, amount={}, 
            is_spent={}, address={:?}, kind={:?})",
            self.transaction_id, self.block_id, self.index, self.amount, self.is_spent, self.address, self.kind
        )
    }
}

impl From<AddressOutputRust> for AddressOutput {
    fn from(ouput: AddressOutputRust) -> Self {
        Self {
            transaction_id: ouput.transaction_id().clone(),
            block_id: ouput.block_id().clone(),
            index: ouput.index().clone(),
            amount: ouput.amount().clone(),
            is_spent: ouput.is_spent().clone(),
            address: ouput.address().clone(),
            kind: ouput.kind().clone(),
        }
    }
}

impl AddressOutput {
    /// The output identifier.
    pub fn id(&self) -> crate::Result<OutputId> {
        OutputId::new(self.transaction_id, self.index).map_err(Into::into)
    }

    pub fn set_transaction_id(&mut self, transaction_id: TransactionId) {
        self.transaction_id = transaction_id
    }
    pub fn set_block_id(&mut self, block_id: BlockId) {
        self.block_id = block_id
    }
    pub fn set_index(&mut self, index: u16) {
        self.index = index
    }
    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount
    }
    pub fn set_spent(&mut self, is_spent: bool) {
        self.is_spent = is_spent
    }

    pub fn address(&self) -> AddressWrapper {
        self.address.clone()
    }
    pub fn set_address(&mut self, address: AddressWrapper) {
        self.address = address
    }

    pub fn kind(&self) -> OutputKind {
        self.kind.clone()
    }
    pub fn set_kind(&mut self, kind: OutputKind) {
        self.kind = kind
    }

    pub fn to_inner(self) -> AddressOutputRust {
        AddressOutputRust {
            transaction_id: self.transaction_id,
            block_id: self.block_id,
            index: self.index,
            amount: self.amount,
            is_spent: self.is_spent,
            address: self.address,
            kind: self.kind.into(),
        }
    }
}

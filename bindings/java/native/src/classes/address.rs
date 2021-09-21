// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    cell::RefCell,
    rc::Rc,
    fmt::{Display, Formatter},
};

use iota_wallet::address::{Address as AddressRust, AddressBuilder as AddressBuilderRust, AddressWrapper};

use crate::Result;

#[derive(Clone, PartialEq, Debug)]
pub struct Address {
    address: AddressRust,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({})", self.readable())
    }
}

impl From<AddressRust> for Address {
    fn from(address: AddressRust) -> Self {
        Self { address }
    }
}

impl Address {
    pub fn builder() -> AddressBuilder {
        AddressBuilder::new()
    }
    
    pub fn readable(&self) -> String {
        self.address.address().to_bech32()
    }

    pub fn balance(&self) -> u64 {
        self.address.balance()
    }

    pub fn to_inner(self) -> AddressRust {
        // TODO: Find a way to not need clone
        self.address.clone()
    }

    pub fn address(&self) -> AddressWrapper {
        self.address.address().clone()
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
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .address(address);
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn key_index(&self, key_index: usize) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .key_index(key_index);
        AddressBuilder::new_with_builder(new_builder)
    }

    /*pub fn outputs(&self, outputs: Vec<AddressOutput>) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .outputs(outputs);
        AddressBuilder::new_with_builder(new_builder)
    }*/

    pub fn internal(&self, internal: bool) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .internal(internal);
        AddressBuilder::new_with_builder(new_builder)
    }

    pub fn build(&self) -> Result<Address> {
        match self.builder.borrow_mut().take().unwrap().build() {
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
            Ok(address) => Ok(Address { address }),
        }
    }
}
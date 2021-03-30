// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter};

use iota_wallet::{
    account::Account,
    address::{Address as AddressRust, AddressOutput, AddressWrapper},
};

#[derive(Clone, PartialEq)]
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
    pub fn readable(&self) -> String {
        self.address.address().to_bech32()
    }

    pub fn balance(&self) -> u64 {
        self.address.balance()
    }

    /// Gets the list of outputs that aren't spent or pending.
    pub fn available_outputs(&self, account: &Account) -> Vec<&AddressOutput> {
        self.address.available_outputs(account)
    }

    pub fn to_inner(self) -> AddressRust {
        // TODO: Find a way to not need clone
        self.address.clone()
    }

    pub fn address(&self) -> AddressWrapper {
        self.address.address().clone()
    }
}

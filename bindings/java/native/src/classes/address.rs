use std::fmt::{
    Display, Formatter
};

use iota_wallet::{
    address::{
        Address as AddressRust,
        AddressOutput, AddressWrapper,
    },
    account::Account,
};

#[derive(Clone, PartialEq)]
pub struct Address {
    address: AddressRust
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({})", self.readable())
    }
}
    
impl Address {
    pub fn new_with_internal(addr: AddressRust) -> Self {
        Address {
            address: addr,
        }
    }

    pub fn readable(&self) -> String {
        self.address.address().to_bech32()
    }

    pub fn balance(&self) -> u64 {
        *self.address.balance()
    }

    /// Gets the list of outputs that aren't spent or pending.
    pub fn available_outputs(&self, account: &Account) -> Vec<&AddressOutput> {
        self.address.available_outputs(account)
    }

    pub fn get_internal(self) -> AddressRust {
        // TODO: Find a way to not need clone
        self.address.clone()
    }

    pub fn address(&self) -> AddressWrapper {
        self.address.address().clone()
    }
}

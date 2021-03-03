use iota_wallet::{
    address::{
        Address as AddressRust,
    },
};

use iota_wallet::account::Account;
use iota_wallet::address::{
    AddressOutput, AddressWrapper
};

pub struct Address {
    address: AddressRust
}

impl Clone for Address {
    fn clone(&self) -> Self {
        Address {
            address: self.address.clone()
        }
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
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
use iota_wallet::{
    address::{
        Address as AddressRust,
    },
};

use iota_wallet::account::Account;
use iota_wallet::address::AddressOutput;

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
/*
    #[getset(set = "pub")]
    balance: u64,
    #[getset(set = "pub(crate)")]
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    internal: bool,
    /// The address outputs.
    #[getset(set = "pub(crate)")]
    pub(crate) outputs: Vec<AddressOutput>,*/

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
}
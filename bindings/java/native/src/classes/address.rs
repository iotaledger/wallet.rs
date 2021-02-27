use iota_wallet::{
    address::{
        Address as AddressRust,
    },
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
    
impl Address {

    pub fn get_internal(self) -> AddressRust {
        // TODO: Find a way to not need clone
        self.address.clone()
    
    }
}
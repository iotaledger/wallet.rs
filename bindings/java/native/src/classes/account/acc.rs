use std::{
    path::PathBuf,
    time::Duration,
    cell::RefCell,
    rc::Rc,
};

use iota_wallet::{
    account::{
        AccountIdentifier,
        AccountInitialiser as AccountInitialiserRust
    },
    address::Address, message::Message,
    DateTime, Local,
};

use crate::Result;
use crate::{
    acc_manager::{
        AccountSignerType
    },
    client_options::ClientOptions,
};

pub struct AccountInitialiser {
    initialiser:  Rc<RefCell<Option<AccountInitialiserRust>>>,
}

impl AccountInitialiser {
    pub fn new(initialiser: AccountInitialiserRust) -> Self {
        Self {
            initialiser: Rc::new(RefCell::new(Option::from(initialiser)))
        }
    }

    fn new_with_initialiser(initialiser: Rc<RefCell<Option<AccountInitialiserRust>>>) -> Self {
        Self {
            initialiser: initialiser
        }
    }

    pub fn signer_type(&mut self, signer_type_enum: AccountSignerType) -> Self {
        let signer_type = crate::acc_manager::signer_type_enum_to_type(signer_type_enum);
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().signer_type(signer_type);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn alias(&mut self, alias: String) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().alias(alias);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn created_at(&mut self, created_at: DateTime<Local>) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().created_at(created_at);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn messages(&mut self, messages: Vec<Message>) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().messages(messages);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn addresses(&mut self, addresses: Vec<Address>) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().addresses(addresses);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn skip_persistance(&mut self) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().skip_persistance();
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    /*
    pub async fn initialise(&self) -> Result<AccountHandle> {

    }
    */
}
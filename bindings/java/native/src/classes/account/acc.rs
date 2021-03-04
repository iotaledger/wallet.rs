use std::{
    cell::RefCell,
    rc::Rc,
};

use iota_wallet::{
    account::{
        AccountInitialiser as AccountInitialiserRust,
        AccountHandle as AccountHandleRust,
        AccountBalance,
    },
    message::{
        MessageId, MessageType,
    },
    DateTime, Local,
};

use crate::{
    client_options::ClientOptions,
    acc_manager::{
        AccountSignerType
    },
    message::{
        Message, Transfer
    },
    address::Address,
    Result,
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

    pub fn signerType(&mut self, signer_type_enum: AccountSignerType) -> Self {
        let signer_type = crate::acc_manager::signer_type_enum_to_type(signer_type_enum);
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().signer_type(signer_type);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn alias(&mut self, alias: String) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().alias(alias);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn createdAt(&mut self, created_at: DateTime<Local>) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().created_at(created_at);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn messages(&mut self, messages: Vec<Message>) -> Self {
        let rust_msgs = messages.into_iter().map(|m| m.get_internal()).collect();
        
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().messages(rust_msgs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn addresses(&mut self, addresses: Vec<Address>) -> Self {
        let rust_addrs = addresses.into_iter().map(|a| a.get_internal()).collect();

        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().addresses(rust_addrs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn skipPersistance(&mut self) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().skip_persistance();
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn initialise(&self) -> Result<Account> {
        let acc_handle = crate::block_on(async move {
            self.initialiser.borrow_mut().take().unwrap().initialise().await
        }).expect("error initialising account");
        Ok(Account {
            handle: acc_handle
        })
    }
}

pub struct Account {
    handle: AccountHandleRust,
}

impl Account {
    pub fn transfer(&mut self, transfer: Transfer) -> Result<Message> {
        let msg = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                self.handle.transfer(transfer.get_internal()).await
            }).expect("failed creating a transfer");

        Ok(Message::new_with_internal(msg))
    }

    pub fn generateAddress(&self) -> Result<Address> {
        let addr = crate::block_on(async move {
            self.handle.generate_address().await
        }).expect("error initialising account");

        Ok(Address::new_with_internal(addr))
    }

    pub fn getUnusedAddress(&self) -> Result<Address> {
        let addr = crate::block_on(async move {
            self.handle.get_unused_address().await
        }).expect("error in getting unused address");

        Ok(Address::new_with_internal(addr))
    }

    pub fn isLatestAddressUnused(&self) -> Result<bool> {
        let is_unused = crate::block_on(async move {
            self.handle.is_latest_address_unused().await
        }).expect("error checking latest addres usage");

        Ok(is_unused)
    }

    pub fn latestAddress(&self) -> Address {
        let latest_address = crate::block_on(async move {
            self.handle.latest_address().await
        });
        Address::new_with_internal(latest_address)
    }

    pub fn setAlias(&self, alias: String) -> Result<()> {
        crate::block_on(async move {
            self.handle.set_alias(alias).await
        }).expect("failed setting new alias");

        Ok(())
    }

    pub fn setClientOptions(&self, options: ClientOptions) -> Result<()> {
        crate::block_on(async move {
            self.handle.set_client_options(options.get_internal()).await
        }).expect("failed setting new client options");

        Ok(())
    }

    pub fn listMessages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Vec<Message> {
        let msgs = crate::block_on(async move {
            self.handle.list_messages(count, from, message_type).await
        });

        msgs.into_iter().map(|m| Message::new_with_internal(m)).collect()
    }

    pub fn listSpentAddresses(&self) -> Vec<Address> {
        let addrs = crate::block_on(async move {
            self.handle.list_spent_addresses().await
        });

        addrs.into_iter().map(|a| Address::new_with_internal(a)).collect()
    }

    pub fn listUnspentAddresses(&self) -> Vec<Address> {
        let addrs = crate::block_on(async move {
            self.handle.list_unspent_addresses().await
        });

        addrs.into_iter().map(|a| Address::new_with_internal(a)).collect()
    }

    pub fn getMessage(&self, message_id: MessageId) -> Option<Message> {
        let msg = crate::block_on(async move {
            self.handle.get_message(&message_id).await
        });

        match msg {
            None => None,
            Some(x) => Some(Message::new_with_internal(x))
        }
    }

    pub fn alias(&self) -> String {
        crate::block_on(async move {
            self.handle.alias().await
        })
    }

    pub fn balance(&self) -> AccountBalance {
        crate::block_on(async move {
            self.handle.balance().await
        })
    }
}
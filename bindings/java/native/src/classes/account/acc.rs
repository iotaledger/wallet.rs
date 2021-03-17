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

use anyhow::anyhow;

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
        let rust_msgs = messages.into_iter().map(|m| m.get_internal()).collect();
        
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().messages(rust_msgs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn addresses(&mut self, addresses: Vec<Address>) -> Self {
        let rust_addrs = addresses.into_iter().map(|a| a.get_internal()).collect();

        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().addresses(rust_addrs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn skip_persistence(&mut self) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().skip_persistence();
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn initialise(&self) -> Result<Account> {
        let acc_handle_res = crate::block_on(async move {
            self.initialiser.borrow_mut().take().unwrap().initialise().await
        });

        match acc_handle_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(acc_handle) => Ok(Account {
                handle: acc_handle
            }),
        }
    }
}

pub struct Account {
    handle: AccountHandleRust,
}

impl Account {
    pub fn new_with_internal(handle: AccountHandleRust) -> Account {
        Account {
            handle: handle
        }
    }

    pub fn consolidate_outputs(&self) -> Result<Vec<Message>> {
        let msgs_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                self.handle.consolidate_outputs().await
            });

        match msgs_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msgs) => Ok(msgs.into_iter().map(|m| Message::new_with_internal(m)).collect()),
        }
    }

    pub fn transfer(&mut self, transfer: Transfer) -> Result<Message> {
        let msg_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                self.handle.transfer(transfer.get_internal()).await
            });

        match msg_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(Message::new_with_internal(msg)),
        }
    }

    pub fn generate_address(&self) -> Result<Address> {
        let addr_res = crate::block_on(async move {
            self.handle.generate_address().await
        });
        
        match addr_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(addr) => Ok(Address::new_with_internal(addr)),
        }
    }

    pub fn get_unused_address(&self) -> Result<Address> {
        let addr_res = crate::block_on(async move {
            self.handle.get_unused_address().await
        });

        match addr_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(addr) => Ok(Address::new_with_internal(addr)),
        }
    }

    pub fn is_latest_address_unused(&self) -> Result<bool> {
        let is_unused_res = crate::block_on(async move {
            self.handle.is_latest_address_unused().await
        });

        match is_unused_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(is_unused) => Ok(is_unused),
        }
    }

    pub fn latest_address(&self) -> Address {
        let latest_address = crate::block_on(async move {
            self.handle.latest_address().await
        });
        Address::new_with_internal(latest_address)
    }

    pub fn set_alias(&self, alias: String) -> Result<()> {
        let res = crate::block_on(async move {
            self.handle.set_alias(alias).await
        });

        match res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn set_client_options(&self, options: ClientOptions) -> Result<()> {
        let opts = crate::block_on(async move {
            self.handle.set_client_options(options.get_internal()).await
        });

        match opts {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn list_messages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Vec<Message> {
        let msgs = crate::block_on(async move {
            self.handle.list_messages(count, from, message_type).await
        });

        msgs.into_iter().map(|m| Message::new_with_internal(m)).collect()
    }

    pub fn list_spent_addresses(&self) -> Vec<Address> {
        let addrs = crate::block_on(async move {
            self.handle.list_spent_addresses().await
        });

        addrs.into_iter().map(|a| Address::new_with_internal(a)).collect()
    }

    pub fn list_unspent_addresses(&self) -> Vec<Address> {
        let addrs = crate::block_on(async move {
            self.handle.list_unspent_addresses().await
        });

        addrs.into_iter().map(|a| Address::new_with_internal(a)).collect()
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<Message> {
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

    pub fn id(&self) -> String {
        crate::block_on(async move {
            self.handle.id().await
        })
    }

    pub fn created_at(&self) -> DateTime<Local> {
        crate::block_on(async move {
            self.handle.created_at().await
        })
    }

    pub fn last_synced_at(&self) -> Option<DateTime<Local>> {
        crate::block_on(async move {
            self.handle.last_synced_at().await
        })
    }
}

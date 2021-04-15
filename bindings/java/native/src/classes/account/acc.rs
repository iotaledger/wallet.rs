// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use crate::{
    acc_manager::AccountSignerType,
    address::Address,
    client_options::ClientOptions,
    message::{Message, Transfer},
    types::NodeInfoWrapper,
    Result,
};
use iota_wallet::{
    account::{AccountBalance, AccountHandle as AccountHandleRust, AccountInitialiser as AccountInitialiserRust},
    message::{MessageId, MessageType},
    DateTime, Local,
};

use anyhow::anyhow;

pub struct AccountInitialiser {
    initialiser: Rc<RefCell<Option<AccountInitialiserRust>>>,
}

impl AccountInitialiser {
    pub fn new(initialiser: AccountInitialiserRust) -> Self {
        Self {
            initialiser: Rc::new(RefCell::new(Option::from(initialiser))),
        }
    }

    fn new_with_initialiser(initialiser: Rc<RefCell<Option<AccountInitialiserRust>>>) -> Self {
        Self {
            initialiser: initialiser,
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
        let rust_msgs = messages.into_iter().map(|m| m.to_inner()).collect();

        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().messages(rust_msgs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn addresses(&mut self, addresses: Vec<Address>) -> Self {
        let rust_addrs = addresses.into_iter().map(|a| a.to_inner()).collect();

        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().addresses(rust_addrs);
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn skip_persistence(&mut self) -> Self {
        let new_initialiser = self.initialiser.borrow_mut().take().unwrap().skip_persistence();
        AccountInitialiser::new_with_initialiser(Rc::new(RefCell::new(Option::from(new_initialiser))))
    }

    pub fn initialise(&self) -> Result<Account> {
        let acc_handle_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { self.initialiser.borrow_mut().take().unwrap().initialise().await });

        match acc_handle_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(acc_handle) => Ok(Account { handle: acc_handle }),
        }
    }
}

pub struct Account {
    handle: AccountHandleRust,
}

impl From<AccountHandleRust> for Account {
    fn from(handle: AccountHandleRust) -> Self {
        Self { handle }
    }
}

impl Account {
    pub fn consolidate_outputs(&self) -> Result<Vec<Message>> {
        let msgs_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { self.handle.consolidate_outputs().await });

        match msgs_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msgs) => Ok(msgs.into_iter().map(|m| m.into()).collect()),
        }
    }

    pub fn get_node_info(&self, url: Option<&str>) -> Result<NodeInfoWrapper> {
        let msgs_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { self.handle.get_node_info(url).await });

        match msgs_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(info) => Ok(info.into()),
        }
    }

    pub fn transfer(&mut self, transfer: Transfer) -> Result<Message> {
        let msg_res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { self.handle.transfer(transfer.to_inner()).await });

        match msg_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(msg.into()),
        }
    }

    pub fn generate_address(&self) -> Result<Address> {
        let addr_res = crate::block_on(async move { self.handle.generate_address().await });

        match addr_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(addr) => Ok(addr.into()),
        }
    }

    pub fn get_unused_address(&self) -> Result<Address> {
        let addr_res = crate::block_on(async move { self.handle.get_unused_address().await });

        match addr_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(addr) => Ok(addr.into()),
        }
    }

    pub fn is_latest_address_unused(&self) -> Result<bool> {
        let is_unused_res = crate::block_on(async move { self.handle.is_latest_address_unused().await });

        match is_unused_res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(is_unused) => Ok(is_unused),
        }
    }

    pub fn latest_address(&self) -> Address {
        let latest_address = crate::block_on(async move { self.handle.latest_address().await });
        latest_address.into()
    }

    pub fn set_alias(&self, alias: String) -> Result<()> {
        let res = crate::block_on(async move { self.handle.set_alias(alias).await });

        match res {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn set_client_options(&self, options: ClientOptions) -> Result<()> {
        let opts = crate::block_on(async move { self.handle.set_client_options(options.to_inner()).await });

        match opts {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn list_messages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Result<Vec<Message>> {
        let msgs = crate::block_on(async move { self.handle.list_messages(count, from, message_type).await })?;
        Ok(msgs.into_iter().map(|m| m.into()).collect())
    }

    pub fn list_spent_addresses(&self) -> Result<Vec<Address>> {
        let addrs = crate::block_on(async move { self.handle.list_spent_addresses().await })?;
        Ok(addrs.into_iter().map(|a| a.into()).collect())
    }

    pub fn list_unspent_addresses(&self) -> Result<Vec<Address>> {
        let addrs = crate::block_on(async move { self.handle.list_unspent_addresses().await })?;
        Ok(addrs.into_iter().map(|a| a.into()).collect())
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<Message> {
        let msg = crate::block_on(async move { self.handle.get_message(&message_id).await });

        match msg {
            None => None,
            Some(m) => Some(m.into()),
        }
    }

    pub fn alias(&self) -> String {
        crate::block_on(async move { self.handle.alias().await })
    }

    pub fn balance(&self) -> Result<AccountBalance> {
        crate::block_on(async move { self.handle.balance().await.map_err(|e| anyhow!(e.to_string())) })
    }

    pub fn id(&self) -> String {
        crate::block_on(async move { self.handle.id().await })
    }

    pub fn created_at(&self) -> DateTime<Local> {
        crate::block_on(async move { self.handle.created_at().await })
    }

    pub fn last_synced_at(&self) -> Option<DateTime<Local>> {
        crate::block_on(async move { self.handle.last_synced_at().await })
    }
}

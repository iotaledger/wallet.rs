  
use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    address::Address,
    message::{
        Message, Transfer,
    },
    Result,
};

use iota_wallet::{
    account::{
        AccountSynchronizer as AccountSynchronizerRust,
        SyncedAccount as SyncedAccountRust,
    },
};

pub struct AccountSynchronizer {
    synchroniser: Rc<RefCell<Option<AccountSynchronizerRust>>>,
}

impl AccountSynchronizer {

    pub fn new_with_instance(synchroniser: AccountSynchronizerRust) -> Self {
        AccountSynchronizer {
            synchroniser: Rc::new(RefCell::new(Option::from(synchroniser))),
        }
    }

    pub fn gap_limit(&mut self, limit: usize) -> Self {
        let new_synchroniser = self.synchroniser.borrow_mut().take().unwrap().gap_limit(limit);
        AccountSynchronizer::new_with_instance(new_synchroniser)
    }

    /// Skip saving new messages and addresses on the account object.
    /// The found data is returned on the `execute` call but won't be persisted on the database.
    pub fn skip_persistance(&mut self) -> Self {
        let new_synchroniser = self.synchroniser.borrow_mut().take().unwrap().skip_persistance();
        AccountSynchronizer::new_with_instance(new_synchroniser)
    }

    /// Initial address index to start syncing.
    pub fn address_index(&mut self, address_index: usize) -> Self {
        let new_synchroniser = self.synchroniser.borrow_mut().take().unwrap().address_index(address_index);
        AccountSynchronizer::new_with_instance(new_synchroniser)
    }

    pub fn execute(&mut self) -> Result<SyncedAccount> {
        let synced_account = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                self.synchroniser.borrow_mut().take().unwrap().execute().await
            }).expect("error syncing account");

        Ok(SyncedAccount {
            synced_account: synced_account
        })
    }
}
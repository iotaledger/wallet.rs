// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use crate::{address::Address, Result};

use iota_wallet::{
    account::{AccountSynchronizer as AccountSynchronizerRust, SyncedAccount as SyncedAccountRust},
    account_manager::AccountsSynchronizer as AccountsSynchronizerRust,
};

use anyhow::anyhow;

pub struct AccountSynchronizer {
    synchroniser: Rc<RefCell<Option<AccountSynchronizerRust>>>,
}

impl From<AccountSynchronizerRust> for AccountSynchronizer {
    fn from(handle: AccountSynchronizerRust) -> Self {
        AccountSynchronizer::new_with_instance(handle)
    }
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
    pub fn skip_persistence(&mut self) -> Self {
        let new_synchroniser = self.synchroniser.borrow_mut().take().unwrap().skip_persistence();
        AccountSynchronizer::new_with_instance(new_synchroniser)
    }

    /// Initial address index to start syncing.
    pub fn address_index(&mut self, address_index: usize) -> Self {
        let new_synchroniser = self
            .synchroniser
            .borrow_mut()
            .take()
            .unwrap()
            .address_index(address_index);
        AccountSynchronizer::new_with_instance(new_synchroniser)
    }

    pub fn execute(&mut self) -> Result<SyncedAccount> {
        let synced_account = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { self.synchroniser.borrow_mut().take().unwrap().execute().await });
        match synced_account {
            Ok(synced_account) => Ok(synced_account.into()),
            Err(e) => Err(anyhow!(e)),
        }
    }
}

pub struct SyncedAccount {
    synced_account: SyncedAccountRust,
}

impl From<SyncedAccountRust> for SyncedAccount {
    fn from(synced_account: SyncedAccountRust) -> Self {
        Self { synced_account }
    }
}

impl SyncedAccount {
    pub fn deposit_address(&mut self) -> Address {
        self.synced_account.deposit_address().clone().into()
    }
}

pub struct AccountsSynchronizer {
    synchroniser: Rc<RefCell<Option<AccountsSynchronizerRust>>>,
}

impl From<AccountsSynchronizerRust> for AccountsSynchronizer {
    fn from(handle: AccountsSynchronizerRust) -> Self {
        AccountsSynchronizer::new_with_instance(handle)
    }
}

impl AccountsSynchronizer {
    pub fn new_with_instance(synchroniser: AccountsSynchronizerRust) -> Self {
        AccountsSynchronizer {
            synchroniser: Rc::new(RefCell::new(Option::from(synchroniser))),
        }
    }

    pub fn gap_limit(&mut self, limit: usize) -> Self {
        let new_synchroniser = self.synchroniser.borrow_mut().take().unwrap().gap_limit(limit);
        AccountsSynchronizer::new_with_instance(new_synchroniser)
    }

    /// Initial address index to start syncing.
    pub fn address_index(&mut self, address_index: usize) -> Self {
        let new_synchroniser = self
            .synchroniser
            .borrow_mut()
            .take()
            .unwrap()
            .address_index(address_index);
        AccountsSynchronizer::new_with_instance(new_synchroniser)
    }

    pub fn execute(&mut self) -> Result<Vec<SyncedAccount>> {
        let synced_accounts = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { self.synchroniser.borrow_mut().take().unwrap().execute().await });
        match synced_accounts {
            Ok(synced_accounts) => Ok(synced_accounts
                .into_iter()
                .map(|synced_account| synced_account.into())
                .collect()),
            Err(e) => Err(anyhow!(e)),
        }
    }
}

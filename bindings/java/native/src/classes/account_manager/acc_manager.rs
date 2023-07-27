// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use crypto::keys::bip39::Mnemonic;
use iota_wallet::{
    account_manager::{
        AccountManager as AccountManagerRust, AccountManagerBuilder as AccountManagerBuilderRust,
        MigrationDataFinder as RustMigrationDataFinder,
    },
    message::MessageId,
    signing::SignerType,
};
use std::{
    cell::RefCell,
    num::NonZeroU64,
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

use crate::{
    acc::{Account, AccountInitialiser},
    client_options::ClientOptions,
    message::Message,
    sync::AccountsSynchronizer,
    types::{MigrationAddress, MigrationBundle, MigrationBundleOptions, MigrationData},
    Result,
};

#[derive(Debug)]
pub enum AccountSignerType {
    Stronghold = 1,
    LedgerNano = 2,
    LedgerNanoSimulator = 3,
}

pub fn signer_type_enum_to_type(signer_type: AccountSignerType) -> SignerType {
    match signer_type {
        #[cfg(feature = "stronghold")]
        AccountSignerType::Stronghold => SignerType::Stronghold,

        #[cfg(feature = "ledger-nano")]
        AccountSignerType::LedgerNano => SignerType::LedgerNano,

        #[cfg(feature = "ledger-nano-simulator")]
        AccountSignerType::LedgerNanoSimulator => SignerType::LedgerNanoSimulator,

        // Default will only happen when we compile without any features...
        #[cfg(not(all(feature = "stronghold", feature = "ledger-nano", feature = "ledger-nano-simulator",)))]
        _ => panic!("No signer type found during compilation"),
    }
}

pub struct AccountManagerBuilder {
    builder: Rc<RefCell<Option<AccountManagerBuilderRust>>>,
}

impl AccountManagerBuilder {
    pub fn new() -> Self {
        AccountManagerBuilder::new_with_builder(AccountManagerBuilderRust::new())
    }

    fn new_with_builder(builder: AccountManagerBuilderRust) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder))),
        }
    }

    pub fn with_storage(&mut self, storage_path: PathBuf, password: Option<&str>) -> Result<Self> {
        match self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_storage(storage_path, password)
        {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(new_builder) => Ok(AccountManagerBuilder::new_with_builder(new_builder)),
        }
    }

    pub fn with_polling_interval(&mut self, polling_interval: Duration) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_polling_interval(polling_interval);
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    /// Sets the number of outputs an address must have to trigger the automatic consolidation process.
    pub fn with_output_consolidation_threshold(&mut self, threshold: usize) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_output_consolidation_threshold(threshold);
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    /// Skip polling
    pub fn with_skip_polling(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_skip_polling();
        AccountManagerBuilder::new_with_builder(new_builder)
    }
    /// Disables the automatic output consolidation process.
    pub fn with_automatic_output_consolidation_disabled(&mut self) -> Self {
        let new_builder = self
            .builder
            .borrow_mut()
            .take()
            .unwrap()
            .with_automatic_output_consolidation_disabled();
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    /// Enables fetching spent output history on sync.
    pub fn with_sync_spent_outputs(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_sync_spent_outputs();
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    /// Enables event persistence.
    pub fn with_event_persistence(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_event_persistence();
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    pub fn with_multiple_empty_accounts(&mut self) -> Self {
        let new_builder = self.builder.borrow_mut().take().unwrap().with_multiple_empty_accounts();
        AccountManagerBuilder::new_with_builder(new_builder)
    }

    /// Builds the manager.
    pub fn finish(&mut self) -> Result<AccountManager> {
        match crate::block_on(async move { self.builder.borrow_mut().take().unwrap().finish().await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(manager) => Ok(AccountManager { manager }),
        }
    }
}

pub struct AccountManager {
    manager: AccountManagerRust,
}

impl AccountManager {
    pub fn storage_path(&self) -> &Path {
        self.manager.storage_path()
    }

    pub fn start_background_sync(
        &mut self,
        polling_interval: Duration,
        automatic_output_consolidation: bool,
    ) -> Result<()> {
        match crate::block_on(self.manager.start_background_sync(
            polling_interval,
            automatic_output_consolidation,
            None,
        )) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn stop_background_sync(&mut self) -> Result<()> {
        self.manager
            .stop_background_sync()
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }

    pub fn set_storage_password(&mut self, password: &str) -> Result<()> {
        match crate::block_on(async move { self.manager.set_storage_password(password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn set_stronghold_password(&mut self, password: &str) -> Result<()> {
        match crate::block_on(async move { self.manager.set_stronghold_password(password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn change_stronghold_password(&mut self, current_password: &str, new_password: &str) -> Result<()> {
        match crate::block_on(async move {
            self.manager
                .change_stronghold_password(current_password, new_password)
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn is_latest_address_unused(&mut self) -> Result<bool> {
        match crate::block_on(async move { self.manager.is_latest_address_unused().await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(b) => Ok(b),
        }
    }

    pub fn set_client_options(&mut self, options: ClientOptions) -> Result<()> {
        match crate::block_on(async move { self.manager.set_client_options(options.to_inner()).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn generate_mnemonic(&mut self) -> Result<String> {
        match self.manager.generate_mnemonic() {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(mnemonic) => Ok(mnemonic),
        }
    }

    pub fn store_mnemonic(&mut self, signer_type_enum: AccountSignerType, mnemonic: Mnemonic) -> Result<()> {
        let signer_type = signer_type_enum_to_type(signer_type_enum);
        let opt_mnemonic = match mnemonic.as_str() {
            "" => None,
            _ => Some(mnemonic),
        };

        match crate::block_on(async move { self.manager.store_mnemonic(signer_type, opt_mnemonic).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn verify_mnemonic(&mut self, mnemonic: String) -> Result<()> {
        match self.manager.verify_mnemonic(mnemonic) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn create_account(&self, client_options: ClientOptions) -> Result<AccountInitialiser> {
        match self.manager.create_account(client_options.to_inner()) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(initialiser) => Ok(AccountInitialiser::new(initialiser)),
        }
    }

    pub fn remove_account(&self, account_id: String) -> Result<()> {
        match crate::block_on(async move { self.manager.remove_account(account_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn get_account(&self, account_id: String) -> Result<Account> {
        match crate::block_on(async move { self.manager.get_account(account_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(acc) => Ok(acc.into()),
        }
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        match crate::block_on(async move { self.manager.get_accounts().await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(accs) => Ok(accs.iter().map(|acc| acc.clone().into()).collect()),
        }
    }

    pub fn sync_accounts(&self) -> Result<AccountsSynchronizer> {
        match self.manager.sync_accounts() {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(s) => Ok(s.into()),
        }
    }

    pub fn reattach(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.reattach(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(msg.into()),
        }
    }

    pub fn promote(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.promote(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(msg.into()),
        }
    }

    pub fn retry(&self, account_id: String, message_id: MessageId) -> Result<Message> {
        match crate::block_on(async move { self.manager.retry(account_id, &message_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(msg.into()),
        }
    }

    pub fn internal_transfer(&self, from_account_id: String, to_account_id: String, amount: u64) -> Result<Message> {
        match crate::block_on(async move {
            self.manager
                .internal_transfer(from_account_id, to_account_id, NonZeroU64::new(amount).unwrap())
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(msg) => Ok(msg.into()),
        }
    }

    pub fn backup(&self, destination: PathBuf, stronghold_password: String) -> Result<PathBuf> {
        match crate::block_on(async move { self.manager.backup(destination, stronghold_password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(path) => Ok(path),
        }
    }

    pub fn import_accounts(&mut self, source: PathBuf, stronghold_password: String) -> Result<()> {
        match crate::block_on(async move { self.manager.import_accounts(source, stronghold_password).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    pub fn get_migration_data(
        &self,
        nodes: Vec<String>,
        seed: &str,
        permanode: Option<&str>,
        security_level: i8,
        initial_address_index: i64,
        gap_limit: i64,
    ) -> Result<MigrationData> {
        let nodes_arr: Vec<&str> = nodes
            .iter()
            .map(|s| {
                let st: &str = &s;
                st
            })
            .collect();
        match RustMigrationDataFinder::new(nodes_arr.as_slice(), seed) {
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
            Ok(mut finder) => {
                if let Some(permanode) = permanode {
                    finder = finder.with_permanode(permanode);
                }
                if initial_address_index > -1 {
                    finder = finder.with_initial_address_index(initial_address_index as u64);
                }
                if gap_limit > -1 {
                    finder = finder.with_gap_limit(gap_limit as u64);
                }
                if security_level > 0 {
                    finder = finder.with_security_level(security_level as u8);
                }
                match crate::block_on(async move { self.manager.get_migration_data(finder).await }) {
                    Err(e) => Err(anyhow!(e.to_string())),
                    Ok(data) => Ok(data.into()),
                }
            }
        }
    }

    pub fn get_migration_address(&self, ledger_prompt: bool, account_id: String) -> Result<MigrationAddress> {
        match crate::block_on(async move { self.manager.get_migration_address(ledger_prompt, account_id).await }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(a) => Ok(a.into()),
        }
    }

    pub fn create_migration_bundle(
        &self,
        seed: String,
        input_address_indexes: Vec<u64>,
        options: MigrationBundleOptions,
    ) -> Result<MigrationBundle> {
        match crate::block_on(async move {
            self.manager
                .create_migration_bundle(
                    &seed,
                    &input_address_indexes,
                    options.mine(),
                    Duration::from_secs(options.timeouts()),
                    options.offset().unwrap_or(0),
                    &options.log_file_name().unwrap_or("migration.log".to_string()),
                )
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(a) => Ok(a.into()),
        }
    }

    pub fn send_migration_bundle(&self, nodes: Vec<String>, bundle_hash: String, mwm: u8) -> Result<()> {
        let nodes_arr: Vec<&str> = nodes
            .iter()
            .map(|s| {
                let st: &str = &s;
                st
            })
            .collect();
        let mwm_real = if mwm < 1 { 14 } else { mwm };
        match crate::block_on(async move {
            self.manager
                .send_migration_bundle(&nodes_arr, &bundle_hash, mwm_real)
                .await
        }) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(_) => Ok(()),
        }
    }
}

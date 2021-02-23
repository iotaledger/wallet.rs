// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::*;
use iota::MessageId as RustMessageId;
use iota_wallet::{
    account_manager::{AccountManager as RustAccountManager, ManagerStorage as RustManagerStorage},
    signing::SignerType as RustSingerType,
};
use pyo3::{exceptions, prelude::*};
use std::{
    convert::{Into, TryInto},
    num::NonZeroU64,
    str::FromStr,
    time::Duration,
};

#[pymethods]
impl AccountManager {
    #[new]
    /// The constructor of account manager.
    fn new(
        storage_path: Option<&str>,
        storage: Option<&str>, // 'Stronghold' or 'Sqlite'
        password: Option<&str>,
        polling_interval: Option<u64>,
    ) -> Result<Self> {
        let mut account_manager = RustAccountManager::builder();
        if storage_path.is_some() & storage.is_some() {
            match storage {
                Some("Stronghold") => {
                    account_manager = account_manager.with_storage(
                        storage_path.unwrap_or_else(|| panic!("invalid Stronghold storage path: {:?}", storage_path)),
                        RustManagerStorage::Stronghold,
                        password,
                    )?
                }
                Some("Sqlite") => {
                    account_manager = account_manager.with_storage(
                        storage_path.unwrap_or_else(|| panic!("invalid Sqlite storage path: {:?}", storage_path)),
                        RustManagerStorage::Sqlite,
                        password,
                    )?
                }
                _ => {
                    return Err(Error {
                        error: PyErr::new::<exceptions::PyValueError, _>("Unsupported storage type!"),
                    })
                }
            }
        }
        if let Some(polling_interval) = polling_interval {
            account_manager = account_manager.with_polling_interval(Duration::from_millis(polling_interval));
        }
        let account_manager = crate::block_on(async { account_manager.finish().await })?;
        Ok(AccountManager { account_manager })
    }

    /// Stops the background polling and MQTT monitoring.
    fn stop_background_sync(&mut self) {
        self.account_manager.stop_background_sync();
    }

    /// Sets the password for the stored accounts.
    fn set_storage_password(&mut self, password: &str) -> Result<()> {
        crate::block_on(async { self.account_manager.set_storage_password(password).await })?;
        Ok(())
    }

    /// Sets the stronghold password.
    fn set_stronghold_password(&mut self, password: &str) -> Result<()> {
        crate::block_on(async { self.account_manager.set_stronghold_password(password).await })?;
        Ok(())
    }

    /// Determines whether all accounts has the latest address unused.
    fn is_latest_address_unused(&self) -> Result<bool> {
        Ok(crate::block_on(async {
            self.account_manager.is_latest_address_unused().await
        })?)
    }

    /// Stores a mnemonic for the given signer type.
    /// If the mnemonic is not provided, we'll generate one.
    fn store_mnemonic(&mut self, signer_type: &str, mnemonic: Option<String>) -> Result<()> {
        let signer_type = match signer_type {
            "Stronghold" => RustSingerType::Stronghold,
            "LedgerNano" => RustSingerType::LedgerNano,
            "LedgerNanoSimulator" => RustSingerType::LedgerNanoSimulator,
            _ => RustSingerType::Custom(signer_type.to_string()),
        };
        Ok(crate::block_on(async {
            self.account_manager.store_mnemonic(signer_type, mnemonic).await
        })?)
    }

    /// Generates a new mnemonic.
    fn generate_mnemonic(&mut self) -> Result<String> {
        Ok(self.account_manager.generate_mnemonic()?)
    }

    /// Checks is the mnemonic is valid. If a mnemonic was generated with `generate_mnemonic()`, the mnemonic here
    /// should match the generated.
    fn verify_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        Ok(self.account_manager.verify_mnemonic(mnemonic)?)
    }

    /// Adds a new account.
    fn create_account(&self, client_options: ClientOptions) -> Result<AccountInitialiser> {
        Ok(AccountInitialiser {
            account_initialiser: Some(self.account_manager.create_account(client_options.into())?),
        })
    }

    /// Deletes an account.
    fn remove_account(&self, account_id: &str) -> Result<()> {
        Ok(crate::block_on(async {
            self.account_manager.remove_account(account_id).await
        })?)
    }

    /// Syncs all accounts.
    fn sync_accounts(&self) -> Result<Vec<SyncedAccount>> {
        let synced_accounts = crate::block_on(async { self.account_manager.sync_accounts().await })?;
        Ok(synced_accounts
            .into_iter()
            .map(|account| SyncedAccount {
                synced_account: account,
            })
            .collect())
    }

    /// Transfers an amount from an account to another.
    fn internal_transfer(&self, from_account_id: &str, to_account_id: &str, amount: u64) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_manager
                .internal_transfer(
                    from_account_id,
                    to_account_id,
                    NonZeroU64::new(amount).unwrap_or_else(|| panic!("invalid internal transfer amount: {}", amount)),
                )
                .await?
                .try_into()
        })
    }

    /// Backups the storage to the given destination
    fn backup(&self, destination: &str) -> Result<String> {
        Ok(
            crate::block_on(async { self.account_manager.backup(destination).await })?
                .into_os_string()
                .into_string()
                .unwrap_or_else(|os_string| {
                    panic!(
                        "invalid backup result {:?} with destination: {:?}",
                        os_string, destination
                    )
                }),
        )
    }

    /// Import backed up accounts.
    fn import_accounts(&mut self, source: &str, stronghold_password: &str) -> Result<()> {
        Ok(crate::block_on(async {
            self.account_manager
                .import_accounts(source, stronghold_password.to_string())
                .await
        })?)
    }

    /// Gets the account associated with the given identifier.
    fn get_account(&self, account_id: &str) -> Result<AccountHandle> {
        let account_handle = crate::block_on(async { self.account_manager.get_account(account_id).await })?;
        Ok(AccountHandle { account_handle })
    }

    /// Gets the account associated with the given identifier.
    fn get_accounts(&self) -> Result<Vec<AccountHandle>> {
        let account_handles = crate::block_on(async { self.account_manager.get_accounts().await })?;
        Ok(account_handles
            .into_iter()
            .map(|handle| AccountHandle { account_handle: handle })
            .collect())
    }

    /// Reattaches an unconfirmed transaction.
    fn reattach(&self, account_id: &str, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_manager
                .reattach(account_id, &RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }

    /// Promotes an unconfirmed transaction.
    fn promote(&self, account_id: &str, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_manager
                .promote(account_id, &RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }

    /// Retries an unconfirmed transaction.
    fn retry(&self, account_id: &str, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_manager
                .retry(account_id, &RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }
}

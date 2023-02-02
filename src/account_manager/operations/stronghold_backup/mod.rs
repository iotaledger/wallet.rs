// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod stronghold_snapshot;

use std::{fs, path::PathBuf, sync::atomic::Ordering};

use iota_client::secret::{stronghold::StrongholdSecretManager, SecretManager, SecretManagerDto};
use zeroize::Zeroize;

use self::stronghold_snapshot::{read_data_from_stronghold_snapshot, store_data_to_stronghold};
#[cfg(feature = "storage")]
use crate::account_manager::AccountManagerBuilder;
use crate::account_manager::{AccountHandle, AccountManager};

impl AccountManager {
    /// Backup the account manager data in a Stronghold file
    /// stronghold_password must be the current one when Stronghold is used as SecretManager.
    pub async fn backup(&self, backup_path: PathBuf, mut stronghold_password: String) -> crate::Result<()> {
        log::debug!("[backup] creating a stronghold backup");
        let mut secret_manager = self.secret_manager.write().await;

        let secret_manager_dto = SecretManagerDto::from(&*secret_manager);

        match &mut *secret_manager {
            // Backup with existing stronghold
            SecretManager::Stronghold(stronghold) => {
                stronghold.set_password(&stronghold_password).await?;

                store_data_to_stronghold(self, stronghold, secret_manager_dto).await?;

                // Write snapshot to backup path
                stronghold.write_stronghold_snapshot(Some(&backup_path)).await?;
            }
            // Backup with new stronghold
            _ => {
                // If the SecretManager is not Stronghold we'll create a new one for the backup
                let mut backup_stronghold = StrongholdSecretManager::builder()
                    .password(&stronghold_password)
                    .build(backup_path)?;

                store_data_to_stronghold(self, &mut backup_stronghold, secret_manager_dto).await?;

                // Write snapshot to backup path
                backup_stronghold.write_stronghold_snapshot(None).await?;
            }
        }

        stronghold_password.zeroize();

        Ok(())
    }

    /// Restore a backup from a Stronghold file
    /// Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already
    /// created If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a
    /// mnemonic was stored, it will be gone.
    pub async fn restore_backup(&self, backup_path: PathBuf, mut stronghold_password: String) -> crate::Result<()> {
        log::debug!("[restore_backup] loading stronghold backup");

        if !backup_path.is_file() {
            return Err(crate::Error::Backup("backup path doesn't exist"));
        }

        let mut accounts = self.accounts.write().await;
        // We don't want to overwrite possible existing accounts
        if !accounts.is_empty() {
            return Err(crate::Error::Backup(
                "can't restore backup when there are already accounts",
            ));
        }

        let mut secret_manager = self.secret_manager.as_ref().write().await;
        // Get the current snapshot path if set
        let new_snapshot_path = if let SecretManager::Stronghold(stronghold) = &mut *secret_manager {
            stronghold.snapshot_path.clone()
        } else {
            PathBuf::from("wallet.stronghold")
        };

        // We'll create a new stronghold to load the backup
        let mut new_stronghold = StrongholdSecretManager::builder()
            .password(&stronghold_password)
            .build(backup_path.clone())?;

        let (read_client_options, read_coin_type, read_secret_manager, read_accounts) =
            read_data_from_stronghold_snapshot(&mut new_stronghold).await?;

        // Update AccountManager with read data
        if let Some(read_client_options) = read_client_options {
            *self.client_options.write().await = read_client_options;
        }

        if let Some(read_coin_type) = read_coin_type {
            self.coin_type.store(read_coin_type, Ordering::Relaxed);
        }

        if let Some(mut read_secret_manager) = read_secret_manager {
            // We have to replace the snapshot path with the current one, when building stronghold
            if let SecretManagerDto::Stronghold(stronghold_dto) = &mut read_secret_manager {
                stronghold_dto.snapshot_path = new_snapshot_path.clone().into_os_string().to_string_lossy().into();
            }

            let mut restored_secret_manager = SecretManager::try_from(&read_secret_manager)
                .map_err(|_| crate::Error::Backup("invalid secret_manager"))?;

            if let SecretManager::Stronghold(stronghold) = &mut restored_secret_manager {
                // Copy Stronghold file so the seed is available in the new location
                fs::copy(backup_path, new_snapshot_path)?;

                // Set password to restored secret manager
                stronghold.set_password(&stronghold_password).await?;
            }
            *secret_manager = restored_secret_manager;
        }

        stronghold_password.zeroize();

        if let Some(read_accounts) = read_accounts {
            let client = self.client_options.read().await.clone().finish()?;

            let mut restored_account_handles = Vec::new();
            for account in read_accounts {
                restored_account_handles.push(AccountHandle::new(
                    account,
                    client.clone(),
                    self.secret_manager.clone(),
                    #[cfg(feature = "events")]
                    self.event_emitter.clone(),
                    #[cfg(feature = "storage")]
                    self.storage_manager.clone(),
                ))
            }
            *accounts = restored_account_handles;
        }

        // store new data
        #[cfg(feature = "storage")]
        {
            let account_manager_builder = AccountManagerBuilder::new()
                .with_secret_manager_arc(self.secret_manager.clone())
                .with_storage_path(
                    &self
                        .storage_options
                        .storage_path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .expect("can't convert os string"),
                )
                .with_client_options(self.client_options.read().await.clone())
                .with_coin_type(self.coin_type.load(Ordering::Relaxed));
            // drop secret manager, otherwise we get a deadlock in save_account_manager_data
            drop(secret_manager);
            self.storage_manager
                .lock()
                .await
                .save_account_manager_data(&account_manager_builder)
                .await?;
            // also save account to db
            for account in accounts.iter() {
                account.save(None).await?;
            }
        }

        Ok(())
    }
}

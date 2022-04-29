// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, str::FromStr};

use iota_client::{
    db::DatabaseProvider,
    secret::{SecretManager, SecretManagerDto},
};

use crate::{
    account::Account,
    account_manager::{AccountHandle, AccountManager, AccountManagerBuilder},
    ClientOptions,
};

pub(crate) const CLIENT_OPTIONS_KEY: &str = "client_options";
pub(crate) const SECRET_MANAGER_KEY: &str = "secret_manager";
pub(crate) const ACCOUNTS_KEY: &str = "accounts";

impl AccountManager {
    /// Backup the account manager data in a Stronghold file
    pub async fn backup(&self, backup_path: PathBuf, stronghold_password: String) -> crate::Result<()> {
        log::debug!("[backup] creating a stronghold backup");
        let mut secret_manager = self.secret_manager.write().await;
        let secret_manager_dto = SecretManagerDto::from(&*secret_manager);
        match &mut *secret_manager {
            SecretManager::Stronghold(stronghold) => {
                stronghold.set_password(&stronghold_password).await;
                // todo: validate that the password is correct?

                // Save current data to Stronghold
                let client_options = self.client_options.read().await.to_json()?;
                stronghold
                    .insert(CLIENT_OPTIONS_KEY.as_bytes(), client_options.as_bytes())
                    .await?;

                stronghold
                    .insert(
                        SECRET_MANAGER_KEY.as_bytes(),
                        serde_json::to_string(&secret_manager_dto)?.as_bytes(),
                    )
                    .await?;

                let mut serialized_accounts = Vec::new();
                for account in self.accounts.read().await.iter() {
                    serialized_accounts.push(serde_json::to_string(&*account.read().await)?);
                }
                stronghold
                    .insert(
                        ACCOUNTS_KEY.as_bytes(),
                        serde_json::to_string(&serialized_accounts)?.as_bytes(),
                    )
                    .await?;

                // Get current snapshot_path to set it again after the backup
                let current_snapshot_path = stronghold.snapshot_path.clone();
                stronghold.snapshot_path = Some(backup_path);
                stronghold.write_stronghold_snapshot().await?;
                stronghold.snapshot_path = current_snapshot_path;
            }
            _ => {
                todo!("create a new StrongholdManager only for the backup")
            }
        }

        Ok(())
    }

    /// Restore a backup from a Stronghold file
    /// Replaces client_options, secret_manager, returns an error if accounts were are already created
    pub async fn restore_backup(&mut self, backup_path: PathBuf, stronghold_password: String) -> crate::Result<()> {
        log::debug!("[restore_backup] loading stronghold backup");
        let mut accounts = self.accounts.write().await;
        // We don't want to overwrite possible existing accounts
        if !accounts.is_empty() {
            return Err(crate::Error::BackupError("Accounts already exist"));
        }

        let mut secret_manager = self.secret_manager.as_ref().write().await;
        let mut new_secret_manager = None;
        if let SecretManager::Stronghold(stronghold) = &mut *secret_manager {
            stronghold.set_password(&stronghold_password).await;
            // Get current snapshot_path to set it again after the backup
            let current_snapshot_path = stronghold.snapshot_path.clone();
            // Read backup
            stronghold.snapshot_path = Some(backup_path.to_path_buf());
            stronghold.read_stronghold_snapshot().await?;
            // Get client_options
            let client_options = stronghold.get(CLIENT_OPTIONS_KEY.as_bytes()).await?;
            if let Some(client_options_bytes) = client_options {
                let client_options_string = String::from_utf8(client_options_bytes)
                    .map_err(|_| crate::Error::BackupError("Invalid client_options"))?;
                let client_options: ClientOptions = serde_json::from_str(&client_options_string)?;
                *self.client_options.write().await = client_options;
                log::debug!("[restore_backup] restored client_options");
            }
            // Get secret_manager
            let restored_secret_manager = stronghold.get(SECRET_MANAGER_KEY.as_bytes()).await?;
            if let Some(restored_secret_manager) = restored_secret_manager {
                let secret_manager_string = String::from_utf8(restored_secret_manager)
                    .map_err(|_| crate::Error::BackupError("Invalid secret_manager"))?;
                let restored_secret_manager = SecretManager::from_str(&secret_manager_string)
                    .map_err(|_| crate::Error::BackupError("Invalid secret_manager"))?;
                // todo: set current password?
                new_secret_manager.replace(restored_secret_manager);
                log::debug!("[restore_backup] restored secret_manager");
            }
            let client = self.client_options.read().await.clone().finish().await?;
            // Get accounts
            let restored_accounts = stronghold.get(ACCOUNTS_KEY.as_bytes()).await?;
            if let Some(restored_accounts) = restored_accounts {
                let restored_accounts_string =
                    String::from_utf8(restored_accounts).map_err(|_| crate::Error::BackupError("Invalid accounts"))?;
                let restored_accounts_string: Vec<String> = serde_json::from_str(&restored_accounts_string)?;
                let restored_accounts = restored_accounts_string
                    .into_iter()
                    .map(|a| Ok(serde_json::from_str(&a)?))
                    .collect::<crate::Result<Vec<Account>>>()?;
                let mut restored_account_handles = Vec::new();
                for account in restored_accounts {
                    restored_account_handles.push(AccountHandle::new(
                        account,
                        client.clone(),
                        self.secret_manager.clone(),
                        self.storage_manager.clone(),
                    ))
                }
                log::debug!("[restore_backup] restored accounts");
                *accounts = restored_account_handles;
            }

            // Set snapshot_path back
            stronghold.snapshot_path = current_snapshot_path;
            // Write stronghold so it's available the next time we start
            stronghold.write_stronghold_snapshot().await?;
        } else {
            todo!("allow restoring backups also with other secret managers")
        }

        // Update secret manager
        // todo: handle mnemonic secret manager, we can't use it from restore, because the mnemonic isn't saved
        if let Some(mut new_secret_manager) = new_secret_manager {
            // Set password to restored secret manager
            if let SecretManager::Stronghold(stronghold) = &mut new_secret_manager {
                stronghold.set_password(&stronghold_password).await;
            }
            *secret_manager = new_secret_manager;
        }

        // store new account manager data
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
                        .expect("Can't convert os string"),
                )
                .with_client_options(self.client_options.read().await.clone());
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

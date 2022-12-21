// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_client::secret::SecretManager;

use crate::account_manager::AccountManager;

impl AccountManager {
    /// Sets the Stronghold password
    pub async fn set_stronghold_password(&self, password: &str) -> crate::Result<()> {
        if let SecretManager::Stronghold(stronghold) = &mut *self.secret_manager.write().await {
            stronghold.set_password(password).await?;
        }
        Ok(())
    }

    /// Change the Stronghold password to another one and also re-encrypt the values in the loaded snapshot with it.
    pub async fn change_stronghold_password(&self, current_password: &str, new_password: &str) -> crate::Result<()> {
        if let SecretManager::Stronghold(stronghold) = &mut *self.secret_manager.write().await {
            stronghold.set_password(current_password).await?;
            stronghold.change_password(new_password).await?;
        }
        Ok(())
    }

    /// Sets the Stronghold password clear interval
    pub async fn set_stronghold_password_clear_interval(&self, timeout: Option<Duration>) -> crate::Result<()> {
        if let SecretManager::Stronghold(stronghold) = &mut *self.secret_manager.write().await {
            stronghold.set_timeout(timeout).await;
        }
        Ok(())
    }

    /// Stores a mnemonic into the Stronghold vault
    pub async fn store_mnemonic(&self, mnemonic: String) -> crate::Result<()> {
        if let SecretManager::Stronghold(stronghold) = &mut *self.secret_manager.write().await {
            stronghold.store_mnemonic(mnemonic).await?;
        }
        Ok(())
    }

    /// Clears the Stronghold password from memory.
    pub async fn clear_stronghold_password(&self) -> crate::Result<()> {
        log::debug!("[clear_stronghold_password]");
        let mut secret_manager = self.secret_manager.write().await;
        match &mut *secret_manager {
            SecretManager::Stronghold(stronghold) => stronghold.clear_key().await,
            _ => return Err(crate::client::Error::SecretManagerMismatch.into()),
        }
        Ok(())
    }

    /// Checks if the Stronghold password is available.
    pub async fn is_stronghold_password_available(&self) -> crate::Result<bool> {
        log::debug!("[is_stronghold_password_available]");
        let mut secret_manager = self.secret_manager.write().await;
        match &mut *secret_manager {
            SecretManager::Stronghold(stronghold) => Ok(stronghold.is_key_available().await),
            _ => Err(crate::client::Error::SecretManagerMismatch.into()),
        }
    }
}

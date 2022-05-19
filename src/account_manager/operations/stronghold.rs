// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::secret::SecretManager;

use crate::account_manager::AccountManager;

impl AccountManager {
    /// Sets the Stronghold password
    pub async fn set_stronghold_password(&self, password: &str) -> crate::Result<()> {
        if let SecretManager::Stronghold(stronghold) = &mut *self.secret_manager.write().await {
            stronghold.set_password(password).await;
            let result = stronghold.read_stronghold_snapshot().await;
            if let Err(err) = result {
                // TODO: replace with actual error matching when updated to the new Stronghold version
                if let iota_client::Error::StrongholdProcedureError(ref err_msg) = err {
                    if !err_msg.contains("IOError") {
                        return Err(err.into());
                    }
                }
            }
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
            _ => return Err(iota_client::Error::SecretManagerMismatch.into()),
        }
        Ok(())
    }

    /// Checks if the Stronghold password is available.
    pub async fn is_stronghold_password_available(&self) -> crate::Result<bool> {
        log::debug!("[is_stronghold_password_available]");
        let mut secret_manager = self.secret_manager.write().await;
        match &mut *secret_manager {
            SecretManager::Stronghold(stronghold) => Ok(stronghold.is_key_available().await),
            _ => Err(iota_client::Error::SecretManagerMismatch.into()),
        }
    }
}

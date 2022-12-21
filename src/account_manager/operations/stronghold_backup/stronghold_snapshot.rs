// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::Ordering;

use crate::{
    account::Account,
    account_manager::AccountManager,
    client::{db::DatabaseProvider, secret::SecretManagerDto, stronghold::StrongholdAdapter},
    ClientOptions,
};

pub(crate) const CLIENT_OPTIONS_KEY: &str = "client_options";
pub(crate) const COIN_TYPE_KEY: &str = "coin_type";
pub(crate) const SECRET_MANAGER_KEY: &str = "secret_manager";
pub(crate) const ACCOUNTS_KEY: &str = "accounts";
pub(crate) const BACKUP_SCHEMA_VERSION_KEY: &str = "backup_schema_version";
pub(crate) const BACKUP_SCHEMA_VERSION: u8 = 1;

pub(crate) async fn store_data_to_stronghold(
    account_manager: &AccountManager,
    stronghold: &mut StrongholdAdapter,
    secret_manager_dto: SecretManagerDto,
) -> crate::Result<()> {
    // Set backup_schema_version
    stronghold
        .insert(BACKUP_SCHEMA_VERSION_KEY.as_bytes(), &[BACKUP_SCHEMA_VERSION])
        .await?;

    let client_options = account_manager.client_options.read().await.to_json()?;
    stronghold
        .insert(CLIENT_OPTIONS_KEY.as_bytes(), client_options.as_bytes())
        .await?;

    let coin_type = account_manager.coin_type.load(Ordering::Relaxed);
    stronghold
        .insert(COIN_TYPE_KEY.as_bytes(), &coin_type.to_le_bytes())
        .await?;

    // Only store secret_managers that aren't SecretManagerDto::Mnemonic, because there the Seed can't be serialized, so
    // we can't create the SecretManager again
    match secret_manager_dto {
        SecretManagerDto::Mnemonic(_) => {}
        _ => {
            stronghold
                .insert(
                    SECRET_MANAGER_KEY.as_bytes(),
                    serde_json::to_string(&secret_manager_dto)?.as_bytes(),
                )
                .await?;
        }
    }

    let mut serialized_accounts = Vec::new();
    for account in account_manager.accounts.read().await.iter() {
        serialized_accounts.push(serde_json::to_string(&*account.read().await)?);
    }
    stronghold
        .insert(
            ACCOUNTS_KEY.as_bytes(),
            serde_json::to_string(&serialized_accounts)?.as_bytes(),
        )
        .await?;

    Ok(())
}

pub(crate) async fn read_data_from_stronghold_snapshot(
    stronghold: &mut StrongholdAdapter,
) -> crate::Result<(
    Option<ClientOptions>,
    Option<u32>,
    Option<SecretManagerDto>,
    Option<Vec<Account>>,
)> {
    // Get version
    let version = stronghold.get(BACKUP_SCHEMA_VERSION_KEY.as_bytes()).await?;
    if let Some(version) = version {
        if version[0] != BACKUP_SCHEMA_VERSION {
            return Err(crate::Error::BackupError("invalid backup_schema_version"));
        }
    }

    // Get client_options
    let client_options_bytes = stronghold.get(CLIENT_OPTIONS_KEY.as_bytes()).await?;
    let client_options = if let Some(client_options_bytes) = client_options_bytes {
        let client_options_string =
            String::from_utf8(client_options_bytes).map_err(|_| crate::Error::BackupError("invalid client_options"))?;
        let client_options: ClientOptions = serde_json::from_str(&client_options_string)?;

        log::debug!("[restore_backup] restored client_options {client_options_string}");
        Some(client_options)
    } else {
        None
    };

    // Get coin_type
    let coin_type_bytes = stronghold.get(COIN_TYPE_KEY.as_bytes()).await?;
    let coin_type = if let Some(coin_type_bytes) = coin_type_bytes {
        let coin_type = u32::from_le_bytes(
            coin_type_bytes
                .try_into()
                .map_err(|_| crate::Error::BackupError("invalid coin_type"))?,
        );
        log::debug!("[restore_backup] restored coin_type: {coin_type}");
        Some(coin_type)
    } else {
        None
    };

    // Get secret_manager
    let restored_secret_manager_bytes = stronghold.get(SECRET_MANAGER_KEY.as_bytes()).await?;
    let restored_secret_manager = if let Some(restored_secret_manager) = restored_secret_manager_bytes {
        let secret_manager_string = String::from_utf8(restored_secret_manager)
            .map_err(|_| crate::Error::BackupError("invalid secret_manager"))?;

        log::debug!("[restore_backup] restored secret_manager: {}", secret_manager_string);

        let secret_manager_dto: SecretManagerDto = serde_json::from_str(&secret_manager_string)?;

        Some(secret_manager_dto)
    } else {
        None
    };

    // Get accounts
    let restored_accounts_bytes = stronghold.get(ACCOUNTS_KEY.as_bytes()).await?;
    let restored_accounts = if let Some(restored_accounts) = restored_accounts_bytes {
        let restored_accounts_string =
            String::from_utf8(restored_accounts).map_err(|_| crate::Error::BackupError("invalid accounts"))?;

        log::debug!("[restore_backup] restore accounts: {restored_accounts_string}");

        let restored_accounts_string: Vec<String> = serde_json::from_str(&restored_accounts_string)?;

        let restored_accounts = restored_accounts_string
            .into_iter()
            .map(|a| Ok(serde_json::from_str(&a)?))
            .collect::<crate::Result<Vec<Account>>>()?;

        Some(restored_accounts)
    } else {
        None
    };

    Ok((client_options, coin_type, restored_secret_manager, restored_accounts))
}

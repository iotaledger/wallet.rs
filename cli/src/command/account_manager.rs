// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::prelude::*};

use clap::{Args, Parser, Subcommand};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::{constants::SHIMMER_COIN_TYPE, secret::SecretManager, utils::generate_mnemonic},
    ClientOptions,
};
use log::LevelFilter;

use crate::{error::Error, helper::get_password};

#[derive(Debug, Clone, Parser)]
#[clap(version, long_about = None)]
#[clap(propagate_version = true)]
pub struct AccountManagerCli {
    #[clap(subcommand)]
    pub command: Option<AccountManagerCommand>,
    pub account: Option<String>,
    #[clap(short, long)]
    pub log_level: Option<LevelFilter>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AccountManagerCommand {
    /// Create a stronghold backup file.
    Backup { path: String },
    /// Change the stronghold password.
    ChangePassword,
    /// Parameters for the init command.
    Init(InitParameters),
    /// Generate a random mnemonic.
    Mnemonic,
    /// Create a new account with an optional alias.
    New { alias: Option<String> },
    /// Restore accounts from a stronghold backup file.
    Restore { backup_path: String },
    /// Set the node to use.
    SetNode { url: String },
    /// Sync all accounts.
    Sync,
}

#[derive(Debug, Clone, Args)]
pub struct InitParameters {
    #[clap(short, long)]
    pub mnemonic: Option<String>,
    #[clap(short, long)]
    pub node: Option<String>,
    #[clap(short, long)]
    pub coin_type: Option<u32>,
}

pub async fn backup_command(manager: &AccountManager, path: String, password: &str) -> Result<(), Error> {
    manager.backup(path.clone().into(), password.into()).await?;

    log::info!("Wallet has been backed up to \"{path}\".");

    Ok(())
}

pub async fn change_password_command(manager: &AccountManager, current: &str) -> Result<(), Error> {
    let new = get_password("Stronghold new password", true)?;

    manager.change_stronghold_password(current, &new).await?;

    Ok(())
}

pub async fn init_command(
    secret_manager: SecretManager,
    storage_path: String,
    parameters: InitParameters,
) -> Result<AccountManager, Error> {
    let account_manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        .with_client_options(
            ClientOptions::new().with_node(parameters.node.as_deref().unwrap_or("http://localhost:14265"))?,
        )
        .with_storage_path(&storage_path)
        .with_coin_type(parameters.coin_type.unwrap_or(SHIMMER_COIN_TYPE))
        .finish()
        .await?;

    let mnemonic = match parameters.mnemonic {
        Some(mnemonic) => mnemonic,
        None => generate_mnemonic()?,
    };

    let mut file = File::options().create(true).append(true).open("mnemonic.txt")?;
    // Write mnemonic with new line
    file.write_all(format!("init_command: {mnemonic}\n").as_bytes())?;

    log::info!("IMPORTANT: mnemonic has been written to \"mnemonic.txt\", handle it safely.");
    log::info!(
        "It is the only way to recover your account if you ever forget your password and/or lose the stronghold file."
    );

    if let SecretManager::Stronghold(secret_manager) = &mut *account_manager.get_secret_manager().write().await {
        secret_manager.store_mnemonic(mnemonic).await?;
    } else {
        panic!("cli-wallet only supports Stronghold-backed secret managers at the moment.");
    }
    log::info!("Mnemonic stored successfully");

    Ok(account_manager)
}

pub async fn mnemonic_command() -> Result<(), Error> {
    let mnemonic = generate_mnemonic()?;

    let mut file = File::options().create(true).append(true).open("mnemonic.txt")?;
    // Write mnemonic with new line
    file.write_all(format!("mnemonic_command: {mnemonic}\n").as_bytes())?;

    log::info!("IMPORTANT: mnemonic has been written to \"mnemonic.txt\", handle it safely.");
    log::info!(
        "It is the only way to recover your account if you ever forget your password and/or lose the stronghold file."
    );

    Ok(())
}

pub async fn new_command(manager: &AccountManager, alias: Option<String>) -> Result<String, Error> {
    let mut builder = manager.create_account();

    if let Some(alias) = alias {
        builder = builder.with_alias(alias);
    }

    let account_handle = builder.finish().await?;
    let alias = account_handle.read().await.alias().to_string();

    log::info!("Created account \"{alias}\"");

    Ok(alias)
}

pub async fn restore_command(
    secret_manager: SecretManager,
    storage_path: String,
    backup_path: String,
    password: String,
) -> Result<AccountManager, Error> {
    let account_manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        // Will be overwritten by the backup's value.
        .with_client_options(
            ClientOptions::new().with_node("http://localhost:14265")?, // .with_ignore_node_health(),
        )
        .with_storage_path(&storage_path)
        // Will be overwritten by the backup's value.
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    account_manager.restore_backup(backup_path.into(), password).await?;

    Ok(account_manager)
}

pub async fn set_node_command(manager: &AccountManager, url: String) -> Result<(), Error> {
    manager
        .set_client_options(ClientOptions::new().with_node(&url)?)
        .await?;

    Ok(())
}

pub async fn sync_command(manager: &AccountManager) -> Result<(), Error> {
    let total_balance = manager.sync(None).await?;

    log::info!("Synchronized all accounts: {:?}", total_balance);

    Ok(())
}

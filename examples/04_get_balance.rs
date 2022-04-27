// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 04_get_balance --release
// In this example we sync the account and get the balance
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold secret_manager
    let secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .snapshot_path(PathBuf::from("wallet.stronghold"))
        .build();

    // Create the account manager
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .finish()
        .await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    // Sync and get the balance
    let _account_balance = account.sync(None).await?;
    // If already synced, just get the balance
    let account_balance = account.balance().await?;

    println!("{:?}", account_balance);

    Ok(())
}

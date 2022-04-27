// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 02_generate_address --release
// In this example we will generate an address
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

    let address = account.generate_addresses(1, None).await?;
    println!("Generated address: {}", address[0].address().to_bech32());

    Ok(())
}

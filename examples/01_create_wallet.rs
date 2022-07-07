// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 01_create_wallet --release
// In this example we will create a new wallet
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .try_build(PathBuf::from("wallet.stronghold"))?;
    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = env::var("NONSECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap();

    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic).await?;

    // Create the account manager with the secret_manager and client options
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Create a new account
    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    println!("Generated a new account");

    Ok(())
}

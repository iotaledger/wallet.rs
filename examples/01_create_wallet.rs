// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 01_create_wallet --release
// In this example we will create a new wallet
// Rename `.env.example` to `.env` first

use dotenv::dotenv;
use iota_wallet::{account_manager::AccountManager, signing::stronghold::StrongholdSigner, ClientOptions, Result};

use std::{env, path::Path};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup Stronghold signer
    let storage_path = Path::new("wallet.stronghold");
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    let signer =
        StrongholdSigner::try_new_signer_handle(&env::var("STRONGHOLD_PASSWORD").unwrap(), &storage_path).unwrap();
    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = env::var("NONSECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap();

    // The mnemonic only needs to be stored the first time
    signer
        .lock()
        .await
        .store_mnemonic(&storage_path, mnemonic)
        .await
        .unwrap();

    // Create the account manager with the signer and client options
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();
    let manager = AccountManager::builder(signer)
        .with_client_options(client_options)
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

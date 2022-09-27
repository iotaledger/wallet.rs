// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we generate addresses which will be used later to find inputs.
//! This example uses dotenv, which is not safe for use in production.
//! `cargo run --example 0_generate_addresses --release`.

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use dotenv::dotenv;
use iota_client::constants::SHIMMER_COIN_TYPE;
use iota_wallet::{
    account::types::AccountAddress,
    account_manager::AccountManager,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, Result,
};

const ADDRESS_FILE_NAME: &str = "examples/offline_signing/addresses.json";

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let offline_client = ClientOptions::new();

    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .build(PathBuf::from("examples/offline_signing/offline_signing.stronghold"))?;
    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap();

    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic).await?;

    // Create the account manager with the secret_manager and client options
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .with_client_options(offline_client)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("examples/offline_signing/offline_walletdb")
        .finish()
        .await?;

    // Create a new account
    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    println!("Generated a new account");

    let addresses = account.addresses().await?;

    write_addresses_to_file(ADDRESS_FILE_NAME, addresses)
}

fn write_addresses_to_file<P: AsRef<Path>>(path: P, addresses: Vec<AccountAddress>) -> Result<()> {
    let json = serde_json::to_string_pretty(&addresses)?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{}", json);

    file.write_all(json.as_bytes())?;

    Ok(())
}

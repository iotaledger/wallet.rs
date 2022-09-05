// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example accounts --release

use std::{env, time::Instant};

use dotenv::dotenv;
use iota_client::utils::request_funds_from_faucet;
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let client_options = ClientOptions::new()
        .with_node(&env::var("NODE_URL").unwrap())?
        .with_node_sync_disabled();

    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap())?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "first_account";

    // create first account
    let _first_account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    // create second account
    let account_alias = "second_account";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let accounts = manager.get_accounts().await?;
    for account in accounts {
        let a = account.read().await;
        println!("Accounts: {:#?}", a);
    }

    let addresses = account.generate_addresses(5, None).await?;

    println!(
        "{}",
        request_funds_from_faucet(&env::var("FAUCET_URL").unwrap(), &addresses[0].address().to_bech32()).await?
    );
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}

// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example background_syncing --release

use std::env;

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let client_options = ClientOptions::new().with_node(&env::var("NODE_URL").unwrap())?;

    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap())?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "logger";
    let account = match manager.get_account(account_alias).await {
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

    account.generate_addresses(1, None).await?;

    manager.start_background_syncing(None, None).await?;
    sleep(Duration::from_secs(10)).await;
    manager.stop_background_syncing()?;
    manager.start_background_syncing(None, None).await?;

    Ok(())
}

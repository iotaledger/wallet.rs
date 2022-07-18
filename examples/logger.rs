// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example logger --release

use std::{env, time::Instant};

use fern_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging.
    let logger_output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(logger_output_config).finish();
    logger_init(config).unwrap();

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

    // Get account or create a new one.
    let account_alias = "logger";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it.
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let _address = account.generate_addresses(5, None).await?;

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());

    println!("Balance: {:?}", balance);

    Ok(())
}

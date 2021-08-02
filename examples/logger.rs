// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example logger --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType};
use log::LevelFilter;
use std::time::Instant;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    // Generates a wallet.log file with logs for debugging
    let output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).unwrap();

    let manager = AccountManager::builder()
        .with_storage("./backup", None)?
        .with_skip_polling()
        .finish()
        .await?;
    manager.set_stronghold_password("password").await?;

    // Get account or create a new one
    let account_alias = "logger";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")?
                .build()
                .unwrap();
            manager
                .create_account(client_options)?
                .alias(account_alias)
                .initialise()
                .await?
        }
    };

    let now = Instant::now();
    account.sync().await.execute().await?;
    println!("Syncing took: {:.2?}", now.elapsed());

    println!("Balance: {:?}", account.balance().await?);

    let addresses = account.list_unspent_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let address = account.generate_address().await?;
    println!("Generated a new address: {:?}", address);

    Ok(())
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example logger --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::account_manager::AccountManager;
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

    let account = manager.get_account("Alice").await?;

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

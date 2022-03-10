// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 04_get_balance --release
// In this example we sync the account and get the balance
// Rename `.env.example` to `.env` first

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    signing::stronghold::StrongholdSigner,
    Result,
};

use std::{env, path::Path};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    init_logger("wallet.log", LevelFilter::Debug)?;

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold signer
    let signer =
        StrongholdSigner::try_new_signer_handle(&env::var("STRONGHOLD_PASSWORD").unwrap(), &Path::new("wallet.stronghold")).unwrap();

    // Create the account manager
    let manager = AccountManager::builder(signer).finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    // Sync and get the balance
    let _account_balance = account.sync(None).await?;
    // If already synced, just get the balance
    let account_balance = account.balance().await?;

    println!("{:?}", account_balance);

    Ok(())
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 03_get_funds --release
// In this example we request funds from the faucet to our address
// Rename `.env.example` to `.env` first

use std::env;

use iota_client::request_funds_from_faucet;
use iota_wallet::{account_manager::AccountManager, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv::dotenv().ok();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let address = account.list_addresses().await?;

    let faucet_response =
        request_funds_from_faucet(&env::var("FAUCET_URL").unwrap(), &address[0].address().to_bech32()).await?;

    println!("{}", faucet_response);

    Ok(())
}

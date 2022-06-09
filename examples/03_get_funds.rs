// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 03_get_funds --release
// In this example we request funds from the faucet to our address
// Rename `.env.example` to `.env` first

use iota_client::request_funds_from_faucet;
use iota_wallet::{account_manager::AccountManager, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let address = account.list_addresses().await?;

    let faucet_response = request_funds_from_faucet(
        "http://localhost:14265/api/plugins/faucet/v1/enqueue",
        &address[0].address(),
    )
    .await?;

    println!("{}", faucet_response);

    Ok(())
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 05_transaction --release
// In this example we will send a transaction
// Rename `.env.example` to `.env` first

use std::env;

use iota_wallet::{account_manager::AccountManager, AddressWithAmount, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production.
    dotenv::dotenv().ok();

    // Create the account manager.
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`.
    let account = manager.get_account("Alice").await?;

    // Set the stronghold password.
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Send a transaction with 1 Mi.
    let outputs = vec![AddressWithAmount {
        address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
        amount: 1_000_000,
    }];
    let transaction = account.send_amount(outputs, None).await?;

    println!(
        "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("No block created yet")
    );

    Ok(())
}

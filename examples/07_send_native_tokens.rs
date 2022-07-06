// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 07_send_native_tokens --release
// In this example we will send native tokens
// Rename `.env.example` to `.env` first

use std::{env, str::FromStr};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager, iota_client::bee_block::output::TokenId, AddressNativeTokens, Result,
};
use primitive_types::U256;

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    let outputs = vec![AddressNativeTokens {
        address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
        native_tokens: vec![(
            // Replace with a TokenId that is available in the account
            TokenId::from_str("0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000")?,
            U256::from(10),
        )],
        ..Default::default()
    }];

    let transaction = account.send_native_tokens(outputs, None).await?;

    println!(
        "Transaction: {} Block sent: http://localhost:14265/api/v2/blocks/{}",
        transaction.transaction_id,
        transaction.block_id.expect("No block created yet")
    );

    Ok(())
}

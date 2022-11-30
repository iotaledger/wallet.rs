// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example increase_native_token_supply --release
// In this example we will melt an existing native token with its foundry
// Rename `.env.example` to `.env` first

use std::{env, str::FromStr};

use dotenv::dotenv;
use iota_client::block::output::TokenId;
use iota_wallet::{account_manager::AccountManager, Result, U256};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let balance = account.balance().await?;
    println!("Balance before melting:\n{balance:?}",);

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Replace with a TokenId that is available in the account, the foundry output which minted it, also needs to be
    // available.
    let token_id = TokenId::from_str("0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000")?;

    // Mint some more native tokens
    let mint_amount = U256::from(10);
    let mint_transaction = account
        .increase_native_token_supply(token_id, mint_amount, None, None)
        .await?;

    account
        .retry_transaction_until_included(&mint_transaction.transaction.transaction_id, None, None)
        .await?;

    let balance = account.sync(None).await?;

    println!("Balance after minting:\n{balance:?}",);

    Ok(())
}

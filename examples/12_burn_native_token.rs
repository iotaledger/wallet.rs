// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 12_burn_native_token --release
// In this example we will burn an existing native token, this will not increase the melted supply in the foundry,
// therefore the foundry output is also not required. But this will also make it impossible to destroy the foundry
// output that minted it.
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
    println!("Balance before burning:\n{balance:?}",);

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Replace with a TokenId that is available in the account
    let token_id = TokenId::from_str("0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000")?;

    // Burn a native token
    let burn_amount = U256::from(1);
    let transaction = account.burn_native_token((token_id, burn_amount), None).await?;

    let _ = match transaction.block_id {
        Some(block_id) => account.retry_until_included(&block_id, None, None).await?,
        None => {
            return Err(iota_wallet::Error::BurningOrMeltingFailed(
                "Burn native token transaction failed to submitted".to_string(),
            ));
        }
    };

    let balance = account.sync(None).await?;

    println!("Balance after burning:\n{balance:?}",);

    Ok(())
}

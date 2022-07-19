// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Run: `cargo run --example 11_melt_native_token --release`.
// In this example we will melt an existing native token with its foundry
// Rename `.env.example` to `.env` first

use std::{env, str::FromStr};

use iota_client::block::output::TokenId;
use iota_wallet::{account_manager::AccountManager, Result, U256};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production.
    dotenv::dotenv().ok();

    // Create the account manager.
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`.
    let account = manager.get_account("Alice").await?;

    let balance = account.balance().await?;
    println!("Balance before melting:\n{balance:?}",);

    // Set the stronghold password.
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Replace with a TokenId that is available in the account, the foundry output whcih minted it, also needs to be
    // available.
    let token_id = TokenId::from_str("0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000")?;

    // Melt some of the circulating supply.
    let melt_amount = U256::from(10);
    let transaction = account.melt_native_token((token_id, melt_amount), None).await?;

    let _ = match transaction.block_id {
        Some(block_id) => account.retry_until_included(&block_id, None, None).await?,
        None => {
            return Err(iota_wallet::Error::BurningOrMeltingFailed(
                "Melt native token transaction failed to submitted".to_string(),
            ));
        }
    };

    let balance = account.sync(None).await?;

    println!("Balance after melting:\n{balance:?}",);

    Ok(())
}

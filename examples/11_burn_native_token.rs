// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 11_burn_native_token --release
// In this example we will burn an existing native token without a foundry
// Rename `.env.example` to `.env` first

use std::env;

use dotenv::dotenv;
use iota_client::bee_message::output::TokenId;
use iota_wallet::{account::SyncOptions, account_manager::AccountManager, Result, U256};

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

    let token_id: [u8; TokenId::LENGTH] = hex::decode(
        "08e5ec7dcdd641b0913ba52cd03ec9ea8b256ce2f6c59decf2ff8fa8857b9d724d0200000000000000000000000000000000",
    )
    .unwrap()
    .try_into()
    .unwrap();
    let token_id = TokenId::new(token_id);

    // Burn some of the circulating supply
    let burn_amount = U256::from(10);
    let _ = account.burn_native_token((token_id, burn_amount), None).await?;

    let sync_options = Some(SyncOptions {
        force_syncing: true,
        ..Default::default()
    });

    let balance = account.sync(sync_options).await?;

    println!("-> {}", serde_json::to_string(&balance)?);

    Ok(())
}

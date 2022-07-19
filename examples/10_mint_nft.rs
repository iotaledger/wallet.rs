// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Run: `cargo run --example 10_mint_nft --release`.
// In this example we will mint a native token
// Rename `.env.example` to `.env` first

use std::env;

use iota_wallet::{account_manager::AccountManager, NftOptions, Result};

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

    let nft_options = vec![NftOptions {
        address: Some("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string()),
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
        metadata: Some(b"some nft metadata".to_vec()),
    }];

    let transaction = account.mint_nfts(nft_options, None).await?;

    println!(
        "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("No block created yet")
    );

    Ok(())
}

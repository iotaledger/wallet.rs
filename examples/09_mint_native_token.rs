// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 09_mint_native_token --release
// In this example we will mint a native token
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager, iota_client::bee_message::output::TokenTag, signing::stronghold::StrongholdSigner,
    NativeTokenOptions, Result, U256,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold signer
    let signer = StrongholdSigner::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .snapshot_path(PathBuf::from("wallet.stronghold"))
        .build();

    // Create the account manager
    let manager = AccountManager::builder().with_signer(signer.into()).finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let native_token_options = NativeTokenOptions {
        account_address: None,
        token_tag: TokenTag::new([0u8; 12]),
        circulating_supply: U256::from(100),
        maxium_supply: U256::from(100),
    };

    let transfer_result = account.mint_native_token(native_token_options, None).await?;
    println!(
        "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
        transfer_result.transaction_id,
        transfer_result.message_id.expect("No message created yet")
    );
    Ok(())
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 07_send_native_tokens --release
// In this example we will send native tokens
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf, str::FromStr};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::bee_message::output::TokenId,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    AddressNativeTokens, Result,
};
use primitive_types::U256;

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold secret_manager
    let secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .snapshot_path(PathBuf::from("wallet.stronghold"))
        .build();

    // Create the account manager
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .finish()
        .await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let outputs = vec![AddressNativeTokens {
        address: "atoi1qqv5avetndkxzgr3jtrswdtz5ze6mag20s0jdqvzk4fwezve8q9vk92ryhu".to_string(),
        native_tokens: vec![(
            TokenId::from_str("089292bbb5129efe5e9bd767aa0a789d475b37047d0100000000000000000000000000000000")?,
            U256::from(10),
        )],
        ..Default::default()
    }];

    let transfer_result = account.send_native_tokens(outputs, None).await?;

    println!(
        "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
        transfer_result.transaction_id,
        transfer_result.message_id.expect("No message created yet")
    );

    Ok(())
}

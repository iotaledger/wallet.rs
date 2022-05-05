// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 11_burn_native_token_without_foundry --release
// In this example we will burn an existing native token without a foundry
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf, time::Duration};

use dotenv::dotenv;
use iota_client::bee_message::output::TokenId;
use iota_wallet::{
    account_manager::AccountManager,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    Result, U256,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    // Setup Stronghold secret_manager
    let secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .snapshot_path(PathBuf::from("wallet.stronghold"))
        .build();

    // Create the account manager with the secret_manager and client options
    let client_options = iota_wallet::ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // Create the account manager
    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .finish()
        .await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let token_id: [u8; TokenId::LENGTH] = hex::decode(
        "08e5ec7dcdd641b0913ba52cd03ec9ea8b256ce2f6c59decf2ff8fa8857b9d724d0200000000000000000000000000000000",
    )
    .unwrap()
    .try_into()
    .unwrap();
    let token_id = TokenId::new(token_id);

    // Burn some of the circulating supply
    let burn_amount = U256::from(10);
    let _ = account
        .burn_native_token_without_foundry((token_id, burn_amount), None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::new(15, 0)).await;
    let _balance = account.sync(None).await?;
    let unspent_output = account.list_unspent_outputs().await?;

    println!("-> {}", serde_json::to_string(&unspent_output)?);

    Ok(())
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 06_send_micro_transaction --release
// In this example we will send an amount below the minimum storage deposit
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf};

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    AddressWithMicroAmount, Result,
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

    // Create the account manager
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .finish()
        .await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let outputs = vec![AddressWithMicroAmount {
        address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
        amount: 1,
        return_address: None,
        expiration: None,
    }];

    let transfer_result = account.send_micro_transaction(outputs, None).await?;

    println!(
        "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
        transfer_result.transaction_id,
        transfer_result.message_id.expect("No message created yet")
    );

    Ok(())
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 05_transaction --release
// In this example we will send a transaction
// Rename `.env.example` to `.env` first

use std::{env, path::PathBuf};

use dotenv::dotenv;
use iota_wallet::{account_manager::AccountManager, signing::stronghold::StrongholdSigner, AddressAndAmount, Result};

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

    // Send a transaction with 1 Mi
    let outputs = vec![AddressAndAmount {
        address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
        amount: 1_000_000,
    }];
    let transfer_result = account.send_amount(outputs, None).await?;

    println!(
        "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
        transfer_result.transaction_id,
        transfer_result.message_id.expect("No message created yet")
    );

    Ok(())
}

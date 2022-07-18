// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we send the signed transaction in a block.
//! `cargo run --example 3_send_transaction --release`.

use std::{fs::File, io::prelude::*, path::Path};

use iota_client::api::{SignedTransactionData, SignedTransactionDataDto};
use iota_wallet::{account_manager::AccountManager, Result};

const SIGNED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/signed_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    let signed_transaction_data = read_signed_transaction_from_file(SIGNED_TRANSACTION_FILE_NAME)?;

    // Create the account manager with the secret_manager and client options.
    let manager = AccountManager::builder()
        .with_storage_path("examples/offline_signing/online_walletdb")
        .finish()
        .await?;

    // Create a new account.
    let account = manager.get_account("Alice").await?;

    // Sends offline signed transaction online.
    let result = account.submit_and_store_transaction(signed_transaction_data).await?;

    println!(
        "Transaction sent: https://explorer.iota.org/devnet/block/{}",
        result.transaction_id
    );

    Ok(())
}

fn read_signed_transaction_from_file<P: AsRef<Path>>(path: P) -> Result<SignedTransactionData> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    let dto = serde_json::from_str::<SignedTransactionDataDto>(&json)?;

    Ok(SignedTransactionData::try_from(&dto)?)
}

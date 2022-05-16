// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we sign the prepared transaction.
//! This example uses dotenv, which is not safe for use in production.
//! `cargo run --example 2_sign_transaction --release`.

use std::{
    env,
    fs::File,
    io::{prelude::*, BufWriter},
    path::{Path, PathBuf},
};

use dotenv::dotenv;
use iota_client::{
    api::{PreparedTransactionData, PreparedTransactionDataDto},
    bee_message::{
        payload::{transaction::dto::TransactionPayloadDto, TransactionPayload},
        unlock_block::UnlockBlocks,
    },
    secret::{stronghold::StrongholdSecretManager, SecretManageExt, SecretManager},
};
use iota_wallet::Result;

const PREPARED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/prepared_transaction.json";
const SIGNED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/signed_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
        .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .snapshot_path(PathBuf::from("examples/offline_signing/offline_signing.stronghold"))
        .build();

    // Load snapshot file
    secret_manager.read_stronghold_snapshot().await?;

    let prepared_transaction = read_prepared_transaction_from_file(PREPARED_TRANSACTION_FILE_NAME)?;

    // Signs prepared transaction offline.
    let unlock_blocks = SecretManager::Stronghold(secret_manager)
        .sign_transaction_essence(&prepared_transaction)
        .await?;
    let unlock_blocks = UnlockBlocks::new(unlock_blocks)?;
    let signed_transaction = TransactionPayload::new(prepared_transaction.essence.clone(), unlock_blocks)?;

    println!("Signed transaction.");

    write_signed_transaction_to_file(SIGNED_TRANSACTION_FILE_NAME, signed_transaction)?;

    Ok(())
}

fn read_prepared_transaction_from_file<P: AsRef<Path>>(path: P) -> Result<PreparedTransactionData> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    Ok(PreparedTransactionData::try_from(&serde_json::from_str::<
        PreparedTransactionDataDto,
    >(&json)?)?)
}

fn write_signed_transaction_to_file<P: AsRef<Path>>(path: P, signed_transaction: TransactionPayload) -> Result<()> {
    let json = serde_json::to_string_pretty(&TransactionPayloadDto::from(&signed_transaction))?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{}", json);

    file.write_all(json.as_bytes())?;

    Ok(())
}

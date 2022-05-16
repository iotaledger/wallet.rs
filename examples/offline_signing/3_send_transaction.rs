// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we send the signed transaction in a message.
//! `cargo run --example 3_send_transaction --release`.

use std::{fs::File, io::prelude::*, path::Path};

use iota_client::{
    api::{verify_semantic, PreparedTransactionData, PreparedTransactionDataDto},
    bee_message::{
        payload::{transaction::dto::TransactionPayloadDto, TransactionPayload},
        semantic::ConflictReason,
    },
    Error,
};
use iota_wallet::{account_manager::AccountManager, Result};

const PREPARED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/prepared_transaction.json";
const SIGNED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/signed_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    let signed_transaction_payload = read_signed_transaction_from_file(SIGNED_TRANSACTION_FILE_NAME)?;

    // TODO @thibault-martinez: I don't like that we have to refetch the prepared transaction. Will revisit later.
    let prepared_transaction = read_prepared_transaction_from_file(PREPARED_TRANSACTION_FILE_NAME)?;

    // Create the account manager with the secret_manager and client options
    let manager = AccountManager::builder()
        .with_storage_path("examples/offline_signing/online_walletdb")
        .finish()
        .await?;

    // Create a new account
    let account = manager.get_account("Alice").await?;

    let (local_time, milestone_index) = account.get_time_and_milestone_checked().await?;

    let conflict = verify_semantic(
        &prepared_transaction.inputs_data,
        &signed_transaction_payload,
        milestone_index,
        local_time,
    )?;

    if conflict != ConflictReason::None {
        return Err(Error::TransactionSemantic(conflict).into());
    }

    // Sends offline signed transaction online.
    let result = account.submit_and_store_transaction(signed_transaction_payload).await?;

    println!(
        "Transaction sent: https://explorer.iota.org/devnet/message/{}",
        result.transaction_id
    );

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

fn read_signed_transaction_from_file<P: AsRef<Path>>(path: P) -> Result<TransactionPayload> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    let payload_dto = serde_json::from_str::<TransactionPayloadDto>(&json)?;
    Ok(TransactionPayload::try_from(&payload_dto)?)
}

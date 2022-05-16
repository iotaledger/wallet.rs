// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will get inputs and prepare a transaction
//! `cargo run --example 1_prepare_transaction --features=ledger_nano --release`.
// todo: remove `--features=ledger_nano`

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use iota_client::{
    api::{PreparedTransactionData, PreparedTransactionDataDto},
    bee_message::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            BasicOutputBuilder,
        },
    },
    secret::{ledger_nano::LedgerSecretManager, SecretManager},
};
use iota_wallet::{account::types::AccountAddress, account_manager::AccountManager, ClientOptions, Result};

const ADDRESS_FILE_NAME: &str = "examples/offline_signing/addresses.json";
const PREPARED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/prepared_transaction.json";

#[tokio::main]

async fn main() -> Result<()> {
    // Address to which we want to send the amount.
    let address = "rms1qruzprxum2934lr3p77t96pzlecxv8pjzvtjrzdcgh2f5exa22n6ga0vm69";
    // The amount to send.
    let amount = 1_000_000;

    // Recovers addresses from example `0_address_generation`.
    let addresses = read_addresses_from_file(ADDRESS_FILE_NAME)?;

    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265/")?
        .with_node_sync_disabled();

    // Create the account manager with the secret_manager and client options
    let manager = AccountManager::builder()
        // todo: remove the need of this workaround
        // We provide the ledger nano simulator as secret_manager so it works, but it's not actually used
        .with_secret_manager(SecretManager::LedgerNanoSimulator(LedgerSecretManager::new(true)))
        .with_client_options(client_options.clone())
        .with_storage_path("examples/offline_signing/online_walletdb")
        .finish()
        .await?;

    // Create a new account
    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .with_addresses(addresses)
        .finish()
        .await?;

    // Sync the account to get the outputs for the addresses
    account.sync(None).await?;

    let byte_cost_config = client_options.finish().await?.get_byte_cost_config().await?;
    let outputs = vec![
        BasicOutputBuilder::new_with_amount(1_000_000)?
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                Address::try_from_bech32(address)?.1,
            )))
            .finish_output()?,
    ];
    let prepared_transaction = account
        // .with_output(address, amount)?
        .prepare_transaction(outputs, None, &byte_cost_config)
        .await?;

    println!("Prepared transaction sending {} to {}.", amount, address);

    write_transaction_to_file(PREPARED_TRANSACTION_FILE_NAME, prepared_transaction)
}

fn read_addresses_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<AccountAddress>> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    Ok(serde_json::from_str(&json)?)
}

fn write_transaction_to_file<P: AsRef<Path>>(path: P, prepared_transaction: PreparedTransactionData) -> Result<()> {
    let json = serde_json::to_string_pretty(&PreparedTransactionDataDto::from(&prepared_transaction))?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{}", json);

    file.write_all(json.as_bytes())?;

    Ok(())
}

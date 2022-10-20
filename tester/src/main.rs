// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::{
        constants::SHIMMER_COIN_TYPE,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::fs;

use self::error::Error;

fn process_fixtures(_account_manager: &AccountManager, fixtures: &Value) -> Result<(), Error> {
    println!("{}", fixtures);

    Ok(())
}

fn process_transactions(_account_manager: &AccountManager, transactions: &Value) -> Result<(), Error> {
    println!("{}", transactions);

    Ok(())
}

fn process_tests(_account_manager: &AccountManager, tests: &Value) -> Result<(), Error> {
    println!("{}", tests);

    Ok(())
}

fn process_json(account_manager: &AccountManager, json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        process_fixtures(account_manager, fixtures)?;
    }

    if let Some(transactions) = json.get("transactions") {
        process_transactions(account_manager, transactions)?;
    }

    if let Some(tests) = json.get("tests") {
        process_tests(account_manager, tests)?;
    }

    Ok(())
}

async fn account_manager() -> Result<AccountManager, Error> {
    let mnemonic = "pumpkin actual foster argue normal dizzy sheriff action license hover fossil pink ancient company toilet silver egg chief actress month family dose orange corn";
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(mnemonic)?;

    let client_options = ClientOptions::new()
        .with_node("https://api.testnet.shimmer.network")?
        .with_node_sync_disabled();

    let account_manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    Ok(account_manager)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let account_manager = account_manager().await?;

    let mut dir = fs::read_dir("json").await?;

    for entry in dir.next_entry().await? {
        let content = fs::read_to_string(entry.path()).await?;
        let json: Value = serde_json::from_str(&content)?;

        println!("{:?}", entry.file_name());
        println!("{}", json);
        process_json(&account_manager, json)?;
    }

    Ok(())
}

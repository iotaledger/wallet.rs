// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use fern_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::{
        constants::SHIMMER_COIN_TYPE,
        generate_mnemonic,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::fs;

use wallet_tester::{
    checks::process_checks, context::Context, error::Error, fixtures::process_fixtures,
    transactions::process_transactions,
};

async fn process_json(context: &Context, json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        process_fixtures(context, fixtures).await?;
    }

    if let Some(transactions) = json.get("transactions") {
        process_transactions(context, transactions).await?;
    }

    if let Some(checks) = json.get("checks") {
        process_checks(context, checks).await?;
    }

    Ok(())
}

async fn account_manager() -> Result<AccountManager, Error> {
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(&generate_mnemonic()?)?;

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
    let logger_output_config = LoggerOutputConfigBuilder::new()
        .level_filter(log::LevelFilter::Info)
        .target_exclusions(&["h2", "hyper", "rustls"])
        .color_enabled(true);

    let config = LoggerConfig::build().with_output(logger_output_config).finish();
    logger_init(config)?;

    let account_manager = account_manager().await?;
    account_manager.create_account().finish().await?;
    let protocol_parameters = account_manager.get_accounts().await?[0]
        .client()
        .get_protocol_parameters()?;
    let context = Context {
        account_manager: account_manager,
        protocol_parameters,
    };

    let mut entries = Vec::new();
    let mut dir = fs::read_dir("json").await?;

    for entry in dir.next_entry().await? {
        entries.push(entry);
    }

    for (index, entry) in entries.iter().enumerate() {
        let content = fs::read_to_string(entry.path()).await?;
        let json: Value = serde_json::from_str(&content)?;

        log::info!(
            "Executing test {}/{}: {:?}",
            index + 1,
            entries.len(),
            entry.file_name(),
        );
        log::debug!("{}", json);

        process_json(&context, json).await?;
    }

    Ok(())
}

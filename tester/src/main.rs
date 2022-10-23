// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use fern_logger::{LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{
    account::AccountHandle,
    account_manager::AccountManager,
    iota_client::{
        constants::SHIMMER_COIN_TYPE,
        generate_mnemonic, request_funds_from_faucet,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::{fs, time};
use wallet_tester::{
    checks::process_checks, context::Context, error::Error, fixtures::process_fixtures, steps::process_steps,
};

fn logger_init() -> Result<(), Error> {
    let logger_output_config = LoggerOutputConfigBuilder::new()
        .level_filter(log::LevelFilter::Info)
        .target_exclusions(&["h2", "hyper", "rustls"])
        .color_enabled(true);
    let logger_config = LoggerConfig::build().with_output(logger_output_config).finish();

    fern_logger::logger_init(logger_config)?;

    Ok(())
}

async fn account_manager(mnemonic: Option<String>) -> Result<AccountManager, Error> {
    let mnemonic = if let Some(mnemonic) = mnemonic {
        mnemonic
    } else {
        generate_mnemonic()?
    };
    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(&mnemonic)?);
    let client_options = ClientOptions::new()
        .with_node("https://api.testnet.shimmer.network")?
        .with_node_sync_disabled();
    let account_manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    Ok(account_manager)
}

async fn process_json<'a>(context: &Context<'a>, json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        process_fixtures(context, fixtures).await?;
    }

    if let Some(steps) = json.get("steps") {
        process_steps(context, steps).await?;
    }

    if let Some(checks) = json.get("checks") {
        process_checks(context, checks).await?;
    }

    Ok(())
}

async fn faucet<'a>(mnemonic: String) -> Result<(AccountManager, AccountHandle), Error> {
    let faucet_manager = account_manager(Some(mnemonic)).await?;
    faucet_manager.create_account().finish().await?;
    let faucet_account = &faucet_manager.get_accounts().await?[0];

    let _res = request_funds_from_faucet(
        "https://faucet.testnet.shimmer.network/api/enqueue",
        &faucet_account.addresses().await?[0].address().to_bech32(),
    )
    .await?;

    time::sleep(Duration::from_secs(10)).await;

    faucet_account.sync(None).await?;

    Ok((faucet_manager, faucet_account.clone()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mnemonic = generate_mnemonic()?;
    let (faucet_manager, faucet_account) = faucet(mnemonic).await?;
    let protocol_parameters = faucet_account.client().get_protocol_parameters()?;

    logger_init()?;

    let mut entries = Vec::new();
    let mut dir = fs::read_dir("json").await?;

    while let Some(entry) = dir.next_entry().await? {
        entries.push(entry);
    }

    for (index, entry) in entries.iter().enumerate() {
        let account_manager = account_manager(None).await?;

        let context = Context {
            faucet_manager: &faucet_manager,
            faucet_account: &faucet_account,
            account_manager,
            protocol_parameters: protocol_parameters.clone(),
        };

        let content = fs::read_to_string(entry.path()).await?;
        let json: Value = serde_json::from_str(&content)?;

        log::info!(
            "Executing test {}/{}: {:?}.",
            index + 1,
            entries.len(),
            entry.file_name(),
        );
        log::debug!("{}", json);

        if let Err(err) = process_json(&context, json).await {
            log::error!(
                "Executing test {}/{}: {:?} failed: {}.",
                index + 1,
                entries.len(),
                entry.file_name(),
                err
            );
        }
    }

    Ok(())
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use fern_logger::{LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{
    account::AccountHandle,
    account_manager::AccountManager,
    iota_client::{
        constants::IOTA_COIN_TYPE,
        generate_mnemonic,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::fs;
use wallet_tester::{
    accounts::process_accounts, checks::process_checks, context::Context, error::Error, steps::process_steps,
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
        .with_node("http://127.0.0.1:14265")?
        .with_node_sync_disabled();
    let account_manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        .with_client_options(client_options)
        .with_coin_type(IOTA_COIN_TYPE)
        .finish()
        .await?;

    Ok(account_manager)
}

async fn faucet<'a>(mnemonic: String) -> Result<(AccountManager, AccountHandle), Error> {
    let faucet_manager = account_manager(Some(mnemonic)).await?;
    faucet_manager.create_account().finish().await?;
    let faucet_account = &faucet_manager.get_accounts().await?[0];

    faucet_account.sync(None).await?;

    Ok((faucet_manager, faucet_account.clone()))
}

async fn process_json<'a>(context: &Context<'a>, json: Value) -> Result<(), Error> {
    if let Some(accounts) = json.get("accounts") {
        process_accounts(context, accounts).await?;
    }

    if let Some(steps) = json.get("steps") {
        process_steps(context, steps).await?;
    }

    if let Some(checks) = json.get("checks") {
        process_checks(context, checks).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut error = false;

    // private tangle faucet mnemonic: https://github.com/iotaledger/hornet/blob/develop/private_tangle/private_tangle_keys.md#faucet
    let mnemonic = String::from("average day true meadow dawn pistol near vicious have ordinary sting fetch mobile month ladder explain tornado curious energy orange belt glue surge urban");
    let (faucet_manager, faucet_account) = faucet(mnemonic).await?;
    let protocol_parameters = faucet_account.client().get_protocol_parameters()?;

    logger_init()?;

    let mut paths = Vec::<PathBuf>::new();

    if let Some(path) = args.get(1) {
        paths.push(PathBuf::from(path));
    } else {
        let mut dir = fs::read_dir("tester/json").await?;

        while let Some(entry) = dir.next_entry().await? {
            paths.push(entry.path());
        }
    };

    for (index, path) in paths.iter().enumerate() {
        let account_manager = account_manager(None).await?;

        let context = Context {
            faucet_manager: &faucet_manager,
            faucet_account: &faucet_account,
            account_manager,
            protocol_parameters: protocol_parameters.clone(),
        };

        let content = fs::read_to_string(path).await?;
        let json: Value = serde_json::from_str(&content)?;

        log::info!(
            "Executing test {}/{}: {:?}.",
            index + 1,
            paths.len(),
            path.file_name().unwrap()
        );
        log::debug!("{}", json);

        if let Err(err) = process_json(&context, json).await {
            log::error!(
                "Executing test {}/{}: {:?} failed: {}.",
                index + 1,
                paths.len(),
                path.file_name().unwrap(),
                err
            );
            error = true;
        }
    }

    if error {
        std::process::exit(1);
    }

    Ok(())
}

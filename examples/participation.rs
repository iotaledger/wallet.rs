// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example participation --features=participation --release

use std::{env, str::FromStr};

use dotenv::dotenv;
use iota_client::{node_api::participation::types::EventId, node_manager::node::Node, Url};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(log::LevelFilter::Debug);
    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let client_options = ClientOptions::new()
        .with_node(&env::var("NODE_URL").unwrap())?
        .with_ignore_node_health();

    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap())?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    let event_id = EventId::from_str("0x0344c97dc9cddc47f880fc1934e361636bf83029268f17faaac97c7be3865f7f")?;
    let event_nodes = vec![Node {
        url: Url::parse("http://localhost:14265").map_err(|e| iota_client::Error::UrlError(e.into()))?,
        auth: None,
        disabled: false,
    }];

    manager.register_participation_event(event_id, event_nodes).await?;

    let registered_participation_events = manager.get_participation_events().await?;

    println!("registered events: {registered_participation_events:?}");

    // Update nodes
    manager
        .register_participation_event(
            event_id,
            vec![
                Node {
                    url: Url::parse("http://localhost:14265").map_err(|e| iota_client::Error::UrlError(e.into()))?,
                    auth: None,
                    disabled: false,
                },
                Node {
                    url: Url::parse("http://localhost:14265").map_err(|e| iota_client::Error::UrlError(e.into()))?,
                    auth: None,
                    disabled: false,
                },
            ],
        )
        .await?;

    let event = manager.get_participation_event(event_id).await;
    println!("event: {event:?}");

    manager.deregister_participation_event(event_id).await?;

    let registered_participation_events = manager.get_participation_events().await?;

    println!("registered events: {registered_participation_events:?}");

    // TODO: add participation account methods

    // // Get account or create a new one
    // let account_alias = "participation";
    // let account = match manager.get_account(account_alias).await {
    //     Ok(account) => account,
    //     _ => {
    //         // first we'll create an example account and store it
    //         manager
    //             .create_account()
    //             .with_alias(account_alias.to_string())
    //             .finish()
    //             .await?
    //     }
    // };

    Ok(())
}

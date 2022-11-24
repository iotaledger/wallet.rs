// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example participation --features=participation --release

use std::{env, str::FromStr};

use dotenv::dotenv;
use iota_client::{node_api::participation::types::EventId, node_manager::node::Node, request_funds_from_faucet, Url};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging.
    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(log::LevelFilter::Debug);
    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();

    // This example uses dotenv, which is not safe for use in production.
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

    let event_id = EventId::from_str("0x80f57f6368933b61af9b3d8e1b152cf5d23bf4537f6362778b0a7302a7000d48")?;
    let event_nodes = vec![Node {
        url: Url::parse("http://localhost:14265").map_err(iota_client::Error::UrlError)?,
        auth: None,
        disabled: false,
    }];

    manager.register_participation_event(event_id, event_nodes).await?;

    let registered_participation_events = manager.get_participation_events().await?;

    println!("registered events: {registered_participation_events:?}");

    // Update nodes.
    manager
        .register_participation_event(
            event_id,
            vec![
                Node {
                    url: Url::parse("http://localhost:14265").map_err(iota_client::Error::UrlError)?,
                    auth: None,
                    disabled: false,
                },
                Node {
                    url: Url::parse("http://localhost:14265").map_err(iota_client::Error::UrlError)?,
                    auth: None,
                    disabled: false,
                },
            ],
        )
        .await?;

    let event = manager.get_participation_event(event_id).await;
    println!("event: {event:?}");

    let event_status = manager.get_participation_event_status(&event_id).await?;
    println!("event status: {event_status:?}");

    // manager.deregister_participation_event(event_id).await?;
    // let registered_participation_events = manager.get_participation_events().await?;
    // println!("registered events: {registered_participation_events:?}");

    // Get account or create a new one.
    let account_alias = "participation";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // First we'll create an example account and store it.
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let address = account.addresses().await?;
    let faucet_response =
        request_funds_from_faucet(&env::var("FAUCET_URL").unwrap(), &address[0].address().to_bech32()).await?;
    println!("{}", faucet_response);

    account.sync(None).await?;

    ////////////////////////////////
    //// create voting output or increase voting power
    //// ////////////////////////////

    let transaction = account.increase_voting_power(1000001).await?;
    println!(
        "Increase voting power Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );

    account
        .retry_until_included(&transaction.block_id.expect("no block created yet"), None, None)
        .await?;
    account.sync(None).await?;

    let voting_output = account.get_voting_output().await?.unwrap();
    println!("Voting output: {:?}", voting_output.output);

    ////////////////////////////////
    //// decrease voting power
    //// ////////////////////////////

    // let transaction = account.decrease_voting_power(1).await?;
    // println!(
    //     "Decrease voting power Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
    //     transaction.transaction_id,
    //     &env::var("NODE_URL").unwrap(),
    //     transaction.block_id.expect("no block created yet")
    // );

    // account
    //     .retry_until_included(&transaction.block_id.expect("no block created yet"), None, None)
    //     .await?;
    // account.sync(None).await?;

    ////////////////////////////////
    //// vote
    //// ////////////////////////////

    let transaction = account.vote(event_id, vec![0]).await?;
    println!(
        "Vote Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );
    account
        .retry_until_included(&transaction.block_id.expect("no block created yet"), None, None)
        .await?;
    account.sync(None).await?;

    ////////////////////////////////
    //// get voting overview
    //// ////////////////////////////

    let overview = account.get_participation_overview().await?;
    println!("overview: {overview:?}");

    ////////////////////////////////
    //// stop vote
    //// ////////////////////////////

    let transaction = account.stop_participating(event_id).await?;
    println!(
        "Vote Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );
    account
        .retry_until_included(&transaction.block_id.expect("no block created yet"), None, None)
        .await?;
    account.sync(None).await?;

    ////////////////////////////////
    //// destroy voting output
    //// ////////////////////////////

    // let voting_output = account.get_voting_output().await?;
    // println!("Voting output: {:?}", voting_output.output);

    // // Decrease full amount, there should be no voting output afterwards
    // let transaction = account.decrease_voting_power(voting_output.output.amount()).await?;
    // println!(
    //     "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
    //     transaction.transaction_id,
    //     &env::var("NODE_URL").unwrap(),
    //     transaction.block_id.expect("no block created yet")
    // );

    // account
    //     .retry_until_included(&transaction.block_id.expect("no block created yet"), None, None)
    //     .await?;
    // account.sync(None).await?;

    // assert!(account.get_voting_output().await.is_err());

    Ok(())
}

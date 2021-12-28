// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example participation --features=participation --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::{
    account_manager::AccountManager,
    client::ClientOptionsBuilder,
    message::MessageId,
    participation::types::{Participation, Participations},
    signing::SignerType,
};
use log::LevelFilter;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    // Generates a wallet.log file with logs for debugging
    let output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).unwrap();

    let manager = AccountManager::builder().finish().await.unwrap();
    manager.set_stronghold_password("password").await.unwrap();

    // Get account or create a new one
    let account_alias = "alias";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                .build()
                .unwrap();
            manager
                .create_account(client_options)?
                .alias(account_alias)
                .initialise()
                .await?
        }
    };
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .build()
        .unwrap();
    account.set_client_options(client_options).await?;
    println!("{:?}", account.get_staking_rewards().await?);
    println!("{:?}", manager.get_participation_events().await?);
    println!("{:?}", manager.get_participation_overview().await?);

    // let address = account.generate_address().await?;
    // println!(
    //     "Send iotas from the faucet to {} and press enter after the transaction got confirmed",
    //     address.address().to_bech32()
    // );
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input).unwrap();
    // println!("Sending participation transfers...");
    // let messages = account
    //     .participate(vec![Participation {
    //         event_id: "06a12548272eb51813a02932dec882656cffe3568090f8675675844d4e2ec186".to_string(),
    //         answers: vec![],
    //     }])
    //     .await?;
    // println!(
    //     "Message sent: {:?}",
    //     messages.iter().map(|m| m.id()).collect::<Vec<&MessageId>>()
    // );

    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input).unwrap();
    // println!("Sending stop participation transfers...");
    // let messages = account
    //     .stop_participating(vec![
    //         "06a12548272eb51813a02932dec882656cffe3568090f8675675844d4e2ec186".to_string()
    //     ])
    //     .await?;
    // println!(
    //     "Message sent: {:?}",
    //     messages.iter().map(|m| m.id()).collect::<Vec<&MessageId>>()
    // );

    // for _ in 0..30 {
    //     println!("{:?}", manager.get_participation_overview().await?);
    //     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    // }

    Ok(())
}

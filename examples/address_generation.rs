// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example address_generation --release
// add --features "ledger-nano" for LedgerNano

use iota_wallet::{
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    ClientOptions, Result,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    // init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil")?;

    let manager = AccountManager::builder(signer)
        .with_client_options(client_options)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "logger";

    let account = match manager.get_account(account_alias.to_string()).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let addresses = account.generate_addresses(3, None).await?;
    let mut bech32_addresses = Vec::new();
    for address in addresses {
        bech32_addresses.push(address.address().to_bech32());
    }
    println!("Generated new addresses: {:#?}", bech32_addresses);
    // generate internal addresses because they are used for the remainder
    // let _address = account
    //     .generate_addresses(
    //         2,
    //         Some(AddressGenerationOptions {
    //             internal: true,
    //             ..Default::default()
    //         }),
    //     )
    //     .await?;

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}

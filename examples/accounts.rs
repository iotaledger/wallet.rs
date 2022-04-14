// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example accounts --release

use std::time::Instant;

use iota_client::utils::request_funds_from_faucet;
use iota_wallet::{account_manager::AccountManager, signing::mnemonic::MnemonicSigner, ClientOptions, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new(
        "flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil",
    )?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "first_account";

    // create first account
    let _first_account = match manager.get_account(account_alias).await {
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

    // create second account
    let account_alias = "second_acccount";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let accounts = manager.get_accounts().await?;
    for account in accounts {
        let a = account.read().await;
        println!("Accounts: {:#?}", a);
    }

    let addresses = account.generate_addresses(5, None).await?;

    println!(
        "{}",
        request_funds_from_faucet(
            "http://localhost:14265/api/plugins/faucet/v1/enqueue",
            &addresses[0].address().to_bech32()
        )
        .await?
    );
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}

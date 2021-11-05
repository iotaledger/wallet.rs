// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example event --release
use iota_wallet::{
    account::AccountHandle, account_manager::AccountManager, address::Address, client::ClientOptionsBuilder,
    event::on_balance_change, signing::SignerType, Error, Result,
};
use reqwest::Client;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let stronghold_password = "password".to_string();
    let account_alias = "alice".to_string();
    let node_url = "https://api.lb-1.h.chrysalis-devnet.iota.cafe/".to_string();
    let faucet_url = "https://faucet.chrysalis-devnet.iota.cafe".to_string();

    let account_manager: AccountManager = AccountManager::builder().finish().await?;
    account_manager.set_stronghold_password(stronghold_password).await?;

    // If no account was previously created, we create one. Otherwise, recover from local storage
    // This ensures that the script can be run multiple times
    let account: AccountHandle = match account_manager.get_account(&account_alias).await {
        Ok(account) => account,
        _ => {
            account_manager.store_mnemonic(SignerType::Stronghold, None).await?;

            let client_options = ClientOptionsBuilder::new().with_node(&node_url)?.build()?;
            account_manager
                .create_account(client_options)?
                .alias(account_alias)
                .initialise()
                .await?
        }
    };

    // Possible events are: on_balance_change, on_broadcast, on_confirmation_state_change, on_error,
    // on_migration_progress, on_new_transaction, on_reattachment, on_stronghold_status_change,
    // on_transfer_progress,
    on_balance_change(move |event| {
        println!("BalanceEvent: {:?}", event);
        println!("Press enter to exit");
    })
    .await;

    let address = account.generate_address().await?;
    println!("Requesting funds from the faucet to {}", address.address().to_bech32());
    get_funds(&address, &faucet_url).await?;

    // Wait for event before exit
    let mut exit = String::new();
    std::io::stdin().read_line(&mut exit).unwrap();
    Ok(())
}

// Requests a testnet funds transaction to our generated address
// This API is rate limited: only a request every minute is allowed
async fn get_funds(address: &Address, faucet_url: &str) -> Result<()> {
    let mut body = HashMap::new();
    body.insert("address", address.address().to_bech32());

    let faucet_response = Client::new()
        .post(format!("{}/api/plugins/faucet/enqueue", faucet_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::ClientError(Box::new(e.into())))?;

    println!(
        "{}",
        faucet_response
            .text()
            .await
            .map_err(|e| Error::ClientError(Box::new(e.into())))?
    );

    println!("Requested funds");

    Ok(())
}

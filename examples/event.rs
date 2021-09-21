// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example event --release

use iota_wallet::{
    account_manager::AccountManager, address::Address, client::ClientOptionsBuilder, event::on_balance_change,
    message::MessageId, signing::SignerType, Result,
};
use serde::Deserialize;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = AccountManager::builder().finish().await?;
    manager.set_stronghold_password("password").await?;
    manager.store_mnemonic(SignerType::Stronghold, None).await?;

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .build()?;

    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

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
    get_funds(&address).await?;

    // Wait for event before exit
    let mut exit = String::new();
    std::io::stdin().read_line(&mut exit).unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct FaucetMessageResponse {
    id: String,
}

#[derive(Deserialize)]
struct FaucetResponse {
    data: FaucetMessageResponse,
}

async fn get_funds(address: &Address) -> Result<MessageId> {
    // use the faucet to get funds on the address
    let response = reqwest::get(&format!(
        "https://faucet.chrysalis-devnet.iota.cafe/api?address={}",
        address.address().to_bech32()
    ))
    .await
    .unwrap()
    .json::<FaucetResponse>()
    .await
    .unwrap();
    let faucet_message_id = MessageId::from_str(&response.data.id)?;

    println!("Got funds from faucet, message id: {:?}", faucet_message_id);

    Ok(faucet_message_id)
}

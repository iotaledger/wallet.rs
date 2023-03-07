// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{bee_message::MessageId, bee_rest_api::types::dtos::LedgerInclusionStateDto, Client, ClientBuilder};
use iota_wallet::{
    account_manager::AccountManager, address::Address, client::ClientOptionsBuilder, signing::SignerType, Result,
};
use serde::Deserialize;

use std::{fs, str::FromStr, time::Duration};

#[derive(Deserialize)]
struct FaucetMessageResponse {
    id: String,
}

#[derive(Deserialize)]
struct FaucetResponse {
    data: FaucetMessageResponse,
}

const OUTPUT_CONSOLIDATION_THRESHOLD: usize = 2;

#[tokio::main]
async fn main() -> Result<()> {
    let node_url = "https://api.lb-0.h.chrysalis-devnet.iota.cafe";
    let storage_path = "./storage/consolidation";

    // clear old storage so we can always start fresh
    let _ = fs::remove_dir_all(storage_path);

    // setup the account manager
    let manager = AccountManager::builder()
        .with_storage(storage_path, None)?
        .with_output_consolidation_threshold(OUTPUT_CONSOLIDATION_THRESHOLD)
        .with_automatic_output_consolidation_disabled()
        .finish()
        .await?;
    manager.set_stronghold_password("password").await?;
    manager.store_mnemonic(SignerType::Stronghold, None).await?;

    // create an account
    let client_options = ClientOptionsBuilder::new()
        .with_node(node_url)?
        .with_local_pow(false)
        .build()?;
    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    // create a iota.rs client to use the node API
    let iota_client = ClientBuilder::new()
        .with_node(node_url)
        .unwrap()
        .with_local_pow(false)
        .finish()
        .await
        .unwrap();

    // get the address we're going to use
    let address = account.read().await.addresses().first().unwrap().clone();

    println!("Address {}", address.address().to_bech32());

    for _ in 1..(OUTPUT_CONSOLIDATION_THRESHOLD + 2) {
        let message_id = get_funds(&address).await?;
        wait_for_message_confirmation(&iota_client, message_id).await;
    }

    let messages = account.consolidate_outputs(false).await?;
    println!("MESSAGES {:?}", messages);

    Ok(())
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

async fn wait_for_message_confirmation(client: &Client, message_id: MessageId) {
    loop {
        let metadata = client.get_message().metadata(&message_id).await.unwrap();
        if let Some(state) = &metadata.ledger_inclusion_state {
            if state == &LedgerInclusionStateDto::Included {
                break;
            } else {
                panic!("message wasn't confirmed; {:?}", metadata);
            }
        } else {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

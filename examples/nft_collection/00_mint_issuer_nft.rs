// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example mint_issuer_nft --release
// In this example we will mint the issuer nft.
// Rename `.env.example` to `.env` and run 01_create_wallet.rs before

use std::env;

use dotenv::dotenv;
use iota_client::block::{
    output::{NftId, Output, OutputId},
    payload::transaction::TransactionEssence,
};
use iota_wallet::{account_manager::AccountManager, NftOptions, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    account.sync(None).await?;

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    let nft_options = vec![NftOptions {
        address: None,
        immutable_metadata: Some(b"This NFT will be the issuer from the awesome NFT collection".to_vec()),
        issuer: None,
        metadata: None,
        sender: None,
        tag: None,
    }];

    let transaction = account.mint_nfts(nft_options, None).await?;

    println!("Transaction: {}.", transaction.transaction_id,);
    println!(
        "Block sent: {}/api/core/v2/blocks/{}.",
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );

    let TransactionEssence::Regular(essence) = transaction.payload.essence();
    for (output_index, output) in essence.outputs().iter().enumerate() {
        if let Output::Nft(nft_output) = output {
            // New minted nft id is empty in the output
            if nft_output.nft_id().is_null() {
                let output_id = OutputId::new(transaction.payload.id(), output_index as u16)?;
                let nft_id = NftId::from(output_id);
                println!("New minted NFT id: {nft_id}");
            }
        }
    }

    Ok(())
}

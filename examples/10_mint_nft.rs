// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example mint_nft --release
// In this example we will mint a native token
// Rename `.env.example` to `.env` first

use std::env;

use dotenv::dotenv;
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::block::output::{
        feature::{IssuerFeature, SenderFeature},
        unlock_condition::AddressUnlockCondition,
        Feature, NftId, NftOutputBuilder, UnlockCondition,
    },
    NftOptions, Result,
};

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
        address: Some("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string()),
        sender: None,
        metadata: Some(b"some nft metadata".to_vec()),
        tag: None,
        issuer: None,
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    }];

    let transaction = account.mint_nfts(nft_options, None).await?;

    println!(
        "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );

    // Build nft output manually
    let sender_address = account.addresses().await?[0].address().clone();
    let outputs = vec![
        // address of the owner of the NFT
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())?
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                *sender_address.as_ref(),
            )))
            .add_feature(Feature::Sender(SenderFeature::new(*sender_address.as_ref())))
            .add_immutable_feature(Feature::Issuer(IssuerFeature::new(*sender_address.as_ref())))
            .finish_output(account.client().get_token_supply()?)?,
    ];

    let transaction = account.send(outputs, None).await?;

    println!(
        "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        transaction.transaction_id,
        &env::var("NODE_URL").unwrap(),
        transaction.block_id.expect("no block created yet")
    );

    Ok(())
}

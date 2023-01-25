// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_client::{
    block::output::{NftId, OutputId},
    request_funds_from_faucet,
};
use iota_wallet::{account::AccountHandle, NativeTokenOptions, NftOptions, Result, U256};

mod common;

#[ignore]
#[tokio::test]
async fn mint_and_burn_nft() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_outputs";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = manager.create_account().finish().await?;

    let account_addresses = account.generate_addresses(1, None).await.unwrap();

    request_funds_from_faucet(
        "http://localhost:8091/api/enqueue",
        &account_addresses[0].address().to_bech32(),
    )
    .await?;

    // Wait for faucet transaction
    tokio::time::sleep(Duration::new(5, 0)).await;
    account.sync(None).await?;

    let nft_options = vec![NftOptions {
        address: Some(account_addresses[0].address().to_bech32()),
        sender: None,
        metadata: Some(b"some nft metadata".to_vec()),
        tag: None,
        issuer: None,
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    }];

    let transaction = account.mint_nfts(nft_options, None).await.unwrap();

    let output_id = OutputId::new(transaction.transaction_id, 0u16).unwrap();
    let nft_id = NftId::from(&output_id);

    tokio::time::sleep(Duration::new(5, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    let _ = account.burn_nft(nft_id, None).await.unwrap();
    tokio::time::sleep(Duration::new(5, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn mint_and_decrease_native_token_supply() -> Result<()> {
    let storage_path = "test-storage/mint_and_decrease_native_token_supply";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = manager.create_account().finish().await?;

    let account_addresses = account.generate_addresses(1, None).await.unwrap();
    request_funds_from_faucet(
        "http://localhost:8091/api/enqueue",
        &account_addresses[0].address().to_bech32(),
    )
    .await?;

    // Wait for faucet transaction
    tokio::time::sleep(Duration::new(10, 0)).await;
    account.sync(None).await?;

    // First create an alias output, this needs to be done only once, because an alias can have many foundry outputs
    let transaction = account.create_alias_output(None, None).await?;

    // Wait for transaction to get included
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;

    tokio::time::sleep(Duration::new(5, 0)).await;
    account.sync(None).await?;

    let circulating_supply = U256::from(60i32);
    let native_token_options = NativeTokenOptions {
        alias_id: None,
        circulating_supply,
        maximum_supply: U256::from(100i32),
        foundry_metadata: None,
    };

    let transaction = account.mint_native_token(native_token_options, None).await.unwrap();

    tokio::time::sleep(Duration::new(10, 0)).await;
    let balance = account.sync(None).await.unwrap();

    let search = balance
        .native_tokens
        .iter()
        .find(|token| token.token_id == transaction.token_id && token.available == circulating_supply);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    // Burn some of the circulating supply
    let burn_amount = U256::from(40i32);
    let _ = account
        .decrease_native_token_supply(transaction.token_id, burn_amount, None)
        .await
        .unwrap();

    tokio::time::sleep(Duration::new(10, 0)).await;
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    let search = balance.native_tokens.iter().find(|token| {
        (token.token_id == transaction.token_id) && (token.available == circulating_supply - burn_amount)
    });
    assert!(search.is_some());

    // The burn the rest of the supply
    let melt_amount = circulating_supply - burn_amount;
    let _ = account
        .decrease_native_token_supply(transaction.token_id, melt_amount, None)
        .await
        .unwrap();

    tokio::time::sleep(Duration::new(5, 0)).await;
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    let search = balance
        .native_tokens
        .iter()
        .find(|token| token.token_id == transaction.token_id);
    assert!(search.is_none());

    // Call to run tests in sequence
    destroy_foundry(&account).await?;
    destroy_alias(&account).await?;

    common::tear_down(storage_path)
}

async fn destroy_foundry(account: &AccountHandle) -> Result<()> {
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's burn the first foundry we can find, although we may not find the required alias output so maybe not a good
    // idea
    let foundry_id = *balance.foundries.first().unwrap();

    let _ = account.destroy_foundry(foundry_id, None).await.unwrap();
    tokio::time::sleep(Duration::new(5, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .foundries
        .iter()
        .find(|&balance_foundry_id| *balance_foundry_id == foundry_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    Ok(())
}

async fn destroy_alias(account: &AccountHandle) -> Result<()> {
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's destroy the first alias we can find
    let alias_id = *balance.aliases.first().unwrap();
    println!("alias_id -> {alias_id}");
    let _ = account.destroy_alias(alias_id, None).await.unwrap();
    tokio::time::sleep(Duration::new(5, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .aliases
        .iter()
        .find(|&balance_alias_id| *balance_alias_id == alias_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    Ok(())
}

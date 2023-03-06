// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use iota_client::block::output::{
    unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition},
    NftId, NftOutputBuilder, OutputId, UnlockCondition,
};
use iota_wallet::{account::AccountHandle, NativeTokenOptions, NftOptions, Result, U256};

#[ignore]
#[tokio::test]
async fn mint_and_burn_nft() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_outputs";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = &common::create_accounts_with_funds(&manager, 1).await?[0];

    let nft_options = vec![NftOptions {
        address: Some(account.addresses().await?[0].address().to_bech32()),
        sender: None,
        metadata: Some(b"some nft metadata".to_vec()),
        tag: None,
        issuer: None,
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    }];

    let transaction = account.mint_nfts(nft_options, None).await.unwrap();
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();

    let output_id = OutputId::new(transaction.transaction_id, 0u16).unwrap();
    let nft_id = NftId::from(&output_id);

    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    let transaction = account.burn_nft(nft_id, None).await.unwrap();
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();
    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn mint_and_burn_expired_nft() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_expired_nft";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account_0 = &common::create_accounts_with_funds(&manager, 1).await?[0];
    let account_1 = manager.create_account().finish().await?;

    let token_supply = account_0.client().get_token_supply().await?;

    let amount = 1_000_000;
    let outputs = vec![
        NftOutputBuilder::new_with_amount(amount, NftId::null())?
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *account_0.addresses().await?[0].address().as_ref(),
                )),
                // immediately expired to account_1
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *account_1.addresses().await?[0].address().as_ref(),
                    1,
                )?),
            ])
            .finish_output(token_supply)?,
    ];

    let transaction = account_0.send(outputs, None).await?;
    account_0
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;

    let output_id = OutputId::new(transaction.transaction_id, 0u16)?;
    let nft_id = NftId::from(&output_id);

    account_1.sync(None).await?;
    let transaction = account_1.burn_nft(nft_id, None).await?;
    account_1
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account_1.sync(None).await?;
    // After burning the amount is available on account_1
    assert_eq!(balance.base_coin.available, amount);

    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn mint_and_decrease_native_token_supply() -> Result<()> {
    let storage_path = "test-storage/mint_and_decrease_native_token_supply";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = &common::create_accounts_with_funds(&manager, 1).await?[0];

    // First create an alias output, this needs to be done only once, because an alias can have many foundry outputs
    let transaction = account.create_alias_output(None, None).await?;

    // Wait for transaction to get included
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    account.sync(None).await?;

    let circulating_supply = U256::from(60i32);
    let native_token_options = NativeTokenOptions {
        alias_id: None,
        circulating_supply,
        maximum_supply: U256::from(100i32),
        foundry_metadata: None,
    };

    let mint_transaction = account.mint_native_token(native_token_options, None).await.unwrap();

    account
        .retry_transaction_until_included(&mint_transaction.transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();

    let search = balance
        .native_tokens
        .iter()
        .find(|token| token.token_id == mint_transaction.token_id && token.available == circulating_supply);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    // Melt some of the circulating supply
    let melt_amount = U256::from(40i32);
    let transaction = account
        .decrease_native_token_supply(mint_transaction.token_id, melt_amount, None)
        .await
        .unwrap();

    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    let search = balance.native_tokens.iter().find(|token| {
        (token.token_id == mint_transaction.token_id) && (token.available == circulating_supply - melt_amount)
    });
    assert!(search.is_some());

    // Then melt the rest of the supply
    let melt_amount = circulating_supply - melt_amount;
    let transaction = account
        .decrease_native_token_supply(mint_transaction.token_id, melt_amount, None)
        .await
        .unwrap();

    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    let search = balance
        .native_tokens
        .iter()
        .find(|token| token.token_id == mint_transaction.token_id);
    assert!(search.is_none());

    // Call to run tests in sequence
    destroy_foundry(account).await?;
    destroy_alias(account).await?;

    common::tear_down(storage_path)
}

async fn destroy_foundry(account: &AccountHandle) -> Result<()> {
    let balance = account.sync(None).await?;
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's burn the first foundry we can find, although we may not find the required alias output so maybe not a good
    // idea
    let foundry_id = *balance.foundries.first().unwrap();

    let transaction = account.destroy_foundry(foundry_id, None).await.unwrap();
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
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
    let transaction = account.destroy_alias(alias_id, None).await.unwrap();
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .aliases
        .iter()
        .find(|&balance_alias_id| *balance_alias_id == alias_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    Ok(())
}

#[ignore]
#[tokio::test]
async fn mint_and_burn_native_tokens() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_native_tokens";
    common::setup(storage_path)?;

    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account = &common::create_accounts_with_funds(&manager, 1).await?[0];

    let native_token_amount = U256::from(100);

    let tx = account.create_alias_output(None, None).await?;
    account
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    account.sync(None).await?;

    let mint_tx = account
        .mint_native_token(
            NativeTokenOptions {
                alias_id: None,
                circulating_supply: native_token_amount,
                maximum_supply: native_token_amount,
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    account
        .retry_transaction_until_included(&mint_tx.transaction.transaction_id, None, None)
        .await?;
    account.sync(None).await?;

    let tx = account
        .burn_native_token(mint_tx.token_id, native_token_amount, None)
        .await?;
    account
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await?;

    assert!(balance.native_tokens.is_empty());

    common::tear_down(storage_path)
}

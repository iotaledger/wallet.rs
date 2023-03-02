// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use iota_wallet::{account::SyncOptions, NativeTokenOptions, Result, U256};

#[ignore]
#[tokio::test]
async fn mint_and_increase_native_token_supply() -> Result<()> {
    let storage_path = "test-storage/mint_and_increase_native_token_supply";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account = &common::create_accounts_with_funds(&manager, 1).await?[0];

    let tx = account.create_alias_output(None, None).await?;
    account
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    account.sync(None).await?;

    let mint_tx = account
        .mint_native_token(
            NativeTokenOptions {
                alias_id: None,
                circulating_supply: U256::from(50),
                maximum_supply: U256::from(100),
                foundry_metadata: None,
            },
            None,
        )
        .await?;
    account
        .retry_transaction_until_included(&mint_tx.transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await?;
    assert_eq!(balance.native_tokens.len(), 1);
    assert_eq!(
        balance
            .native_tokens
            .iter()
            .find(|t| t.token_id == mint_tx.token_id)
            .unwrap()
            .available,
        U256::from(50)
    );

    let mint_tx = account
        .increase_native_token_supply(mint_tx.token_id, U256::from(50), None, None)
        .await?;
    account
        .retry_transaction_until_included(&mint_tx.transaction.transaction_id, None, None)
        .await?;
    let balance = account.sync(None).await?;
    assert_eq!(balance.native_tokens.len(), 1);
    assert_eq!(
        balance
            .native_tokens
            .iter()
            .find(|t| t.token_id == mint_tx.token_id)
            .unwrap()
            .available,
        U256::from(100)
    );

    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn native_token_foundry_metadata() -> Result<()> {
    let storage_path = "test-storage/native_token_foundry_metadata";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account = &common::create_accounts_with_funds(&manager, 1).await?[0];

    let tx = account.create_alias_output(None, None).await?;
    account
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;
    account.sync(None).await?;

    let foundry_metadata = vec![1, 3, 3, 7];

    let mint_tx = account
        .mint_native_token(
            NativeTokenOptions {
                alias_id: None,
                circulating_supply: U256::from(50),
                maximum_supply: U256::from(100),
                foundry_metadata: Some(foundry_metadata.clone()),
            },
            None,
        )
        .await?;
    account
        .retry_transaction_until_included(&mint_tx.transaction.transaction_id, None, None)
        .await?;
    // Sync native_token_foundries to get the metadata
    let balance = account
        .sync(Some(SyncOptions {
            sync_native_token_foundries: true,
            ..Default::default()
        }))
        .await?;
    assert_eq!(balance.native_tokens.len(), 1);
    // Metadata should exist and be the same
    assert_eq!(
        balance
            .native_tokens
            .iter()
            .find(|t| t.token_id == mint_tx.token_id)
            .unwrap()
            .metadata
            .as_ref()
            .unwrap()
            .data(),
        &foundry_metadata
    );

    common::tear_down(storage_path)
}

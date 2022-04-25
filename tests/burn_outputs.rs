// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_client::{
    bee_block::output::{NftId, OutputId, TokenTag, AliasId},
    request_funds_from_faucet,
};
use iota_wallet::{
    account_manager::AccountManager, signing::mnemonic::MnemonicSigner, AliasOptions, ClientOptions, Error,
    NativeTokenOptions, NftOptions, Result, U256,
};

#[tokio::test]
async fn mint_and_burn_nft() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_outputs";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    ).unwrap();

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path(storage_path)
        .finish()
        .await
        .unwrap();

    let account = match manager.get_account("Alice".to_string()).await {
        Ok(account) => account,
        Err(Error::AccountNotFound) => manager
            .create_account()
            .with_alias("Alice".to_string())
            .finish()
            .await
            .unwrap(),
        Err(e) => return Err(e),
    };

    let account_addresses = account.generate_addresses(1, None).await.unwrap();

    let faucet_response = request_funds_from_faucet(
        "http://localhost:14265/api/plugins/faucet/v1/enqueue",
        &account_addresses[0].address().to_bech32(),
    )
    .await?;

    println!("{}", faucet_response);

    let nft_options = vec![NftOptions {
        address: Some(account_addresses[0].address().to_bech32()),
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
        metadata: Some(b"some nft metadata".to_vec()),
    }];

    let transfer_result = account.mint_nfts(nft_options, None).await.unwrap();

    let output_id = OutputId::new(transfer_result.transaction_id, 0u16).unwrap();
    let nft_id = NftId::from(output_id);

    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    let _ = account.burn_nft(nft_id, None).await.unwrap();
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance.nfts.iter().find(|&balance_nft_id| *balance_nft_id == nft_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    std::fs::remove_dir_all(storage_path).unwrap_or(());
    assert!(search.is_none());

    Ok(())
}

#[tokio::test]
async fn mint_and_burn_native_token() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_outputs";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    ).unwrap();

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path(storage_path)
        .finish()
        .await
        .unwrap();

    let account = match manager.get_account("Alice".to_string()).await {
        Ok(account) => account,
        Err(Error::AccountNotFound) => manager
            .create_account()
            .with_alias("Alice".to_string())
            .finish()
            .await
            .unwrap(),
        Err(e) => return Err(e),
    };

    let account_addresses = account.generate_addresses(1, None).await.unwrap();

    let faucet_response = request_funds_from_faucet(
        "http://localhost:14265/api/plugins/faucet/v1/enqueue",
        &account_addresses[0].address().to_bech32(),
    )
    .await?;

    println!("{}", faucet_response);

    let circulating_supply = U256::from(60);

    let native_token_options = NativeTokenOptions {
        account_address: Some(account_addresses[0].address().to_bech32()),
        token_tag: TokenTag::new([0u8; 12]),
        circulating_supply,
        maximum_supply: U256::from(100),
    };

    let transfer_result = account.mint_native_token(native_token_options, None).await.unwrap();
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .native_tokens
        .iter()
        .find(|&token| *token.0 == transfer_result.token_id && *token.1 == circulating_supply);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    // Burn some of the circulating supply
    let burn_amount = U256::from(40);
    let _ = account
        .burn_native_token((transfer_result.token_id, burn_amount), None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .native_tokens
        .iter()
        .find(|&token| (*token.0 == transfer_result.token_id) && (*token.1 == circulating_supply - burn_amount));
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_some());

    // The burn the rest of the supply
    let burn_amount = circulating_supply - burn_amount;
    let _ = account
        .burn_native_token((transfer_result.token_id, burn_amount), None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .native_tokens
        .iter()
        .find(|&token| *token.0 == transfer_result.token_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    std::fs::remove_dir_all(storage_path).unwrap_or(());

    Ok(())
}

#[tokio::test]
async fn mint_and_burn_native_token_without_foundry() -> Result<()> {
    Err(Error::BurningFailed("not implemented".to_string()))
}

#[tokio::test]
async fn burn_foundry() -> Result<()> {
    let storage_path = "test-storage/burn_foundry";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    ).unwrap();

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path(storage_path)
        .finish()
        .await
        .unwrap();

    let account = match manager.get_account("Alice".to_string()).await {
        Ok(account) => account,
        Err(Error::AccountNotFound) => manager
            .create_account()
            .with_alias("Alice".to_string())
            .finish()
            .await
            .unwrap(),
        Err(e) => return Err(e),
    };

    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's burn the first foundry we can find
    let foundry_id = balance.foundries.first().unwrap().clone();

    let _ = account.burn_foundry(foundry_id.clone(), None).await?;
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .foundries
        .iter()
        .find(|&balance_foundry_id| *balance_foundry_id == foundry_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    Ok(())
}

#[tokio::test]
async fn destroy_alias() -> Result<()> {
    let storage_path = "test-storage/destroy_alias";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    ).unwrap();

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path(storage_path)
        .finish()
        .await
        .unwrap();

    let account = match manager.get_account("Alice".to_string()).await {
        Ok(account) => account,
        Err(Error::AccountNotFound) => manager
            .create_account()
            .with_alias("Alice".to_string())
            .finish()
            .await
            .unwrap(),
        Err(e) => return Err(e),
    };

    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's destroy the first alias we can find
    // let alias_id = balance.aliases.first().unwrap().clone();
    let alias_id: [u8; 20] = hex::decode("f7d16b94e0c1d0542ce8014a5a16312afaccf391")
        .unwrap()
        .try_into()
        .unwrap();
    let alias_id = AliasId::new(alias_id);
    println!("alias_id -> {alias_id}");
    let alias_options = AliasOptions {
        alias_id,
        burn_foundries: None,
    };
    let _ = account.destroy_alias(alias_options, None).await?;
    tokio::time::sleep(Duration::new(15, 0)).await;
    let balance = account.sync(None).await.unwrap();
    let search = balance
        .aliases
        .iter()
        .find(|&balance_alias_id| *balance_alias_id == alias_id);
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());
    assert!(search.is_none());

    Ok(())
}

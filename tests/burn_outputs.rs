// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_client::{
    bee_block::output::{NftId, OutputId, TokenTag, AliasId},
    request_funds_from_faucet,
};
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    AliasOptions, ClientOptions, Error, NativeTokenOptions, NftOptions, Result, U256,
};

#[tokio::test]
async fn mint_and_burn_nft() -> Result<()> {
    let storage_path = "test-storage/mint_and_burn_outputs";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
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
        "http://faucet.localhost:14265/api/enqueue",
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
        .with_node("http://api.localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
            "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
        )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
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
        "http://faucet.localhost:14265/api/enqueue",
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
        foundry_metadata: None,
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
async fn burn_foundry() -> Result<()> {
    let storage_path = "test-storage/burn_foundry";
    std::fs::remove_dir_all(storage_path).unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")
        .unwrap()
        .with_node_sync_disabled();

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
            "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
        )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
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

    let _account_addresses = account.generate_addresses(1, None).await.unwrap();
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's burn the first foundry we can find, although we may not find the required alias output so maybe not a good idea
    let foundry_id = balance.foundries.first().unwrap().clone();
    // let foundry_id: [u8; FoundryId::LENGTH] = hex::decode("087711e3ffdc9db7ff13c0605996c47f2c8570425b4fbef98ab17119efab31a68e0100000000").unwrap().try_into().unwrap();
    // let foundry_id = FoundryId::new(foundry_id);

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

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
            "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
        )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
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

    let _account_addresses = account.generate_addresses(1, None).await.unwrap();
    let balance = account.sync(None).await.unwrap();
    println!("account balance -> {}", serde_json::to_string(&balance).unwrap());

    // Let's destroy the first alias we can find, this can fail if one of its foundries have been previously burnt
    let alias_id = balance.aliases.first().unwrap().clone();
    // let alias_id: [u8; AliasId::LENGTH] =
    //     hex::decode("e37b17f36e490beb8b7a757cf14e0b1b5006a739b2478ea8a056ef7cafe9e2a3")
    //         .unwrap()
    //         .try_into()
    //         .unwrap();
    // let alias_id = AliasId::new(alias_id);
    println!("alias_id -> {alias_id}");
    let alias_options = AliasOptions {
        alias_id,
        // burn_foundries: false,
        ..Default::default()
    };
    let _ = account.destroy_alias(alias_options, None).await.unwrap();
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

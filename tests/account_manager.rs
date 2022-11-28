// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    constants::{IOTA_COIN_TYPE, SHIMMER_COIN_TYPE},
    node_manager::node::{Node, NodeDto, Url},
};
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::test]
async fn stored_account_manager_data() -> Result<()> {
    std::fs::remove_dir_all("test-storage/stored_account_manager_data").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://some-not-default-node:14265")?;

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/stored_account_manager_data")
        .finish()
        .await?;

    drop(manager);

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    // Recreate AccountManager without providing client options
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_storage_path("test-storage/stored_account_manager_data")
        .finish()
        .await?;
    let client_options = manager.get_client_options().await;

    let node_dto = NodeDto::Node(Node::from(Url::parse("http://some-not-default-node:14265").unwrap()));

    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    std::fs::remove_dir_all("test-storage/stored_account_manager_data").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn different_seed() -> Result<()> {
    std::fs::remove_dir_all("test-storage/different_seed").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://localhost:14265")?;
    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/different_seed")
        .finish()
        .await?;

    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    drop(_account);
    drop(manager);

    // Recreate AccountManager with a different mnemonic
    let secret_manager2 = MnemonicSecretManager::try_from_mnemonic(
        "route hen wink below army inmate object crew vintage gas best space visit say fortune gown few brain emerge umbrella consider spider digital galaxy",
    )?;
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager2))
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/different_seed")
        .finish()
        .await?;

    // Generating a new account needs to return an error, because the seed from the secret_manager is different
    assert!(
        manager
            .create_account()
            .with_alias("Bob".to_string())
            .finish()
            .await
            .is_err()
    );

    std::fs::remove_dir_all("test-storage/different_seed").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn changed_coin_type() -> Result<()> {
    std::fs::remove_dir_all("test-storage/changed_coin_type").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://localhost:14265")?;
    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/changed_coin_type")
        .finish()
        .await?;

    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    drop(_account);
    drop(manager);

    // Recreate AccountManager with same mnemonic
    let secret_manager2 = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager2))
        .with_coin_type(IOTA_COIN_TYPE)
        .with_storage_path("test-storage/changed_coin_type")
        .finish()
        .await?;

    // Generating a new account needs to return an error, because a different coin type was set and we require all
    // accounts to have the same coin type
    assert!(
        manager
            .create_account()
            .with_alias("Bob".to_string())
            .finish()
            .await
            .is_err()
    );

    std::fs::remove_dir_all("test-storage/changed_coin_type").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn shimmer_coin_type() -> Result<()> {
    std::fs::remove_dir_all("test-storage/shimmer_coin_type").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://localhost:14265")?;

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/shimmer_coin_type")
        .finish()
        .await?;

    let account = manager.create_account().finish().await?;

    // Creating a new account with providing a coin type will use the Shimmer coin type with shimmer testnet bech32 hrp
    assert_eq!(
        &account.addresses().await?[0].address().to_bech32(),
        // Address generated with bip32 path: [44, 4219, 0, 0, 0]
        "smr1qq724zgvdujt3jdcd3xzsuqq7wl9pwq3dvsa5zvx49rj9tme8cat65xq7jz"
    );

    std::fs::remove_dir_all("test-storage/shimmer_coin_type").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn iota_coin_type() -> Result<()> {
    std::fs::remove_dir_all("test-storage/iota_coin_type").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://localhost:14265")?;

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(IOTA_COIN_TYPE)
        .with_storage_path("test-storage/iota_coin_type")
        .finish()
        .await?;

    let account = manager.create_account().finish().await?;

    // Creating a new account with providing a coin type will use the iota coin type with shimmer testnet bech32 hrp
    assert_eq!(
        &account.addresses().await?[0].address().to_bech32(),
        // Address generated with bip32 path: [44, 4218, 0, 0, 0]
        "smr1qrpwecegav7eh0z363ca69laxej64rrt4e3u0rtycyuh0mam3vq3ulygj9p"
    );

    std::fs::remove_dir_all("test-storage/iota_coin_type").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn generate_address_shimmer_coin_type() -> Result<()> {
    std::fs::remove_dir_all("test-storage/generate_address_shimmer_coin_type").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_ignore_node_health();

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/generate_address_shimmer_coin_type")
        .finish()
        .await?;

    let address = manager.generate_address(0, false, 0, None).await?;

    // Creating a new account with providing a coin type will use the Shimmer coin type with shimmer testnet bech32 hrp
    assert_eq!(
        &address.to_bech32("smr"),
        // Address generated with bip32 path: [44, 4219, 0, 0, 0]
        "smr1qq724zgvdujt3jdcd3xzsuqq7wl9pwq3dvsa5zvx49rj9tme8cat65xq7jz"
    );

    std::fs::remove_dir_all("test-storage/generate_address_shimmer_coin_type").unwrap_or(());
    Ok(())
}

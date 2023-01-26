// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    constants::IOTA_COIN_TYPE,
    node_manager::node::{Node, NodeDto, Url},
};
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

mod common;

#[tokio::test]
async fn update_client_options() -> Result<()> {
    let storage_path = "test-storage/update_client_options";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, Some(common::NODE_OTHER)).await?;

    let node_dto_old = NodeDto::Node(Node::from(Url::parse(common::NODE_OTHER).unwrap()));
    let node_dto_new = NodeDto::Node(Node::from(Url::parse(common::NODE_LOCAL).unwrap()));

    let client_options = manager.get_client_options().await;
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto_old));
    assert!(!client_options.node_manager_builder.nodes.contains(&node_dto_new));

    manager
        .set_client_options(ClientOptions::new().with_node(common::NODE_LOCAL)?)
        .await?;

    let client_options = manager.get_client_options().await;
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto_new));
    assert!(!client_options.node_manager_builder.nodes.contains(&node_dto_old));

    common::tear_down(storage_path)
}

#[tokio::test]
async fn different_seed() -> Result<()> {
    let storage_path = "test-storage/different_seed";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    drop(_account);
    drop(manager);

    // Recreate AccountManager with a different mnemonic
    let mnemonic = "route hen wink below army inmate object crew vintage gas best space visit say fortune gown few brain emerge umbrella consider spider digital galaxy";
    let manager = common::make_manager(storage_path, Some(mnemonic), None).await?;

    // Generating a new account needs to return an error, because the seed from the secret_manager is different
    assert!(
        manager
            .create_account()
            .with_alias("Bob".to_string())
            .finish()
            .await
            .is_err()
    );

    common::tear_down(storage_path)
}

#[tokio::test]
async fn changed_coin_type() -> Result<()> {
    let storage_path = "test-storage/changed_coin_type";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
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
        .with_storage_path(storage_path)
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

    common::tear_down(storage_path)
}

#[tokio::test]
async fn shimmer_coin_type() -> Result<()> {
    let storage_path = "test-storage/shimmer_coin_type";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = manager.create_account().finish().await?;

    // Creating a new account with providing a coin type will use the Shimmer coin type with shimmer testnet bech32 hrp
    assert_eq!(
        &account.addresses().await?[0].address().as_ref().to_bech32("smr"),
        // Address generated with bip32 path: [44, 4219, 0, 0, 0]
        "smr1qq724zgvdujt3jdcd3xzsuqq7wl9pwq3dvsa5zvx49rj9tme8cat65xq7jz"
    );

    common::tear_down(storage_path)
}

#[tokio::test]
async fn iota_coin_type() -> Result<()> {
    let storage_path = "test-storage/iota_coin_type";
    common::setup(storage_path)?;

    let client_options = ClientOptions::new().with_node(common::NODE_LOCAL)?;
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(common::DEFAULT_MNEMONIC)?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(IOTA_COIN_TYPE)
        .with_storage_path(storage_path)
        .finish()
        .await?;

    let account = manager.create_account().finish().await?;

    // Creating a new account with providing a coin type will use the iota coin type with shimmer testnet bech32 hrp
    assert_eq!(
        &account.addresses().await?[0].address().as_ref().to_bech32("smr"),
        // Address generated with bip32 path: [44, 4218, 0, 0, 0]
        "smr1qrpwecegav7eh0z363ca69laxej64rrt4e3u0rtycyuh0mam3vq3ulygj9p"
    );

    common::tear_down(storage_path)
}

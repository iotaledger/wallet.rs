// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(all(feature = "stronghold", feature = "storage"))]
mod common;

#[cfg(all(feature = "stronghold", feature = "storage"))]
use std::path::PathBuf;

#[cfg(all(feature = "stronghold", feature = "storage"))]
use iota_client::{
    constants::{IOTA_COIN_TYPE, SHIMMER_COIN_TYPE},
    node_manager::node::{Node, NodeDto, Url},
    secret::{mnemonic::MnemonicSecretManager, stronghold::StrongholdSecretManager, SecretManager},
};
#[cfg(all(feature = "stronghold", feature = "storage"))]
use iota_wallet::{account_manager::AccountManager, ClientOptions, Result};

#[tokio::test]
#[cfg(all(feature = "stronghold", feature = "storage"))]
// Backup and restore with Stronghold
async fn backup_and_restore() -> Result<()> {
    let storage_path = "test-storage/backup_and_restore";
    common::setup(storage_path)?;

    let client_options = ClientOptions::new().with_node(common::NODE_LOCAL)?;

    let stronghold_password = "some_hopefully_secure_password";

    // Create directory if not existing, because stronghold panics otherwise
    std::fs::create_dir_all(storage_path).unwrap_or(());
    let mut stronghold = StrongholdSecretManager::builder()
        .password(stronghold_password)
        .build(PathBuf::from("test-storage/backup_and_restore/1.stronghold"))?;

    stronghold.store_mnemonic("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string()).await.unwrap();

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(stronghold))
        .with_client_options(client_options.clone())
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/backup_and_restore/1")
        .finish()
        .await?;

    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    manager
        .backup(
            PathBuf::from("test-storage/backup_and_restore/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;

    // restore from backup

    let stronghold =
        StrongholdSecretManager::builder().build(PathBuf::from("test-storage/backup_and_restore/2.stronghold"))?;

    let restore_manager = AccountManager::builder()
        .with_storage_path("test-storage/backup_and_restore/2")
        .with_secret_manager(SecretManager::Stronghold(stronghold))
        .with_client_options(ClientOptions::new().with_node(common::NODE_OTHER)?)
        // Build with a different coin type, to check if it gets replaced by the one from the backup
        .with_coin_type(IOTA_COIN_TYPE)
        .finish()
        .await?;

    // Wrong password fails
    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore/backup.stronghold"),
            "wrong password".to_string(),
            None,
        )
        .await
        .unwrap_err();

    // Correct password works, even after trying with a wrong one before
    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore/backup.stronghold"),
            stronghold_password.to_string(),
            None,
        )
        .await?;

    // Validate restored data

    // Restored coin type is used
    let new_account = restore_manager.create_account().finish().await?;
    assert_eq!(new_account.read().await.coin_type(), &SHIMMER_COIN_TYPE);

    // compare restored client options
    let client_options = restore_manager.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse(common::NODE_LOCAL).unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    // Get account
    let recovered_account = restore_manager.get_account("Alice").await?;
    assert_eq!(account.addresses().await?, recovered_account.addresses().await?);

    // secret manager is the same
    assert_eq!(
        account.generate_addresses(1, None).await?,
        recovered_account.generate_addresses(1, None).await?
    );
    common::tear_down(storage_path)
}

#[tokio::test]
#[cfg(all(feature = "stronghold", feature = "storage"))]
// Backup and restore with Stronghold and MnemonicSecretManager
async fn backup_and_restore_mnemonic_secret_manager() -> Result<()> {
    let storage_path = "test-storage/backup_and_restore_mnemonic_secret_manager";
    common::setup(storage_path)?;

    let client_options = ClientOptions::new().with_node(common::NODE_LOCAL)?;

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options.clone())
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/backup_and_restore_mnemonic_secret_manager/1")
        .finish()
        .await?;

    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    let stronghold_password = "some_hopefully_secure_password";

    // Create directory if not existing, because stronghold panics otherwise
    std::fs::create_dir_all(storage_path).unwrap_or(());
    manager
        .backup(
            PathBuf::from("test-storage/backup_and_restore_mnemonic_secret_manager/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;

    // restore from backup

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let restore_manager = AccountManager::builder()
        .with_storage_path("test-storage/backup_and_restore_mnemonic_secret_manager/2")
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        // Build with a different coin type, to check if it gets replaced by the one from the backup
        .with_coin_type(IOTA_COIN_TYPE)
        .with_client_options(ClientOptions::new().with_node(common::NODE_OTHER)?)
        .finish()
        .await?;

    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore_mnemonic_secret_manager/backup.stronghold"),
            stronghold_password.to_string(),
            None,
        )
        .await?;

    // Validate restored data

    // Restored coin type is used
    let new_account = restore_manager.create_account().finish().await?;
    assert_eq!(new_account.read().await.coin_type(), &SHIMMER_COIN_TYPE);

    // compare restored client options
    let client_options = restore_manager.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse(common::NODE_LOCAL).unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    // Get account
    let recovered_account = restore_manager.get_account("Alice").await?;
    assert_eq!(account.addresses().await?, recovered_account.addresses().await?);

    // secret manager is the same
    assert_eq!(
        account.generate_addresses(1, None).await?,
        recovered_account.generate_addresses(1, None).await?
    );
    common::tear_down(storage_path)
}

#[tokio::test]
#[cfg(all(feature = "stronghold", feature = "storage"))]
// Backup and restore with Stronghold
async fn backup_and_restore_different_coin_type() -> Result<()> {
    let storage_path = "test-storage/backup_and_restore_different_coin_type";
    common::setup(storage_path)?;

    let client_options = ClientOptions::new().with_node(common::NODE_LOCAL)?;

    let stronghold_password = "some_hopefully_secure_password";

    // Create directory if not existing, because stronghold panics otherwise
    std::fs::create_dir_all(storage_path).unwrap_or(());
    let mut stronghold = StrongholdSecretManager::builder()
        .password(stronghold_password)
        .build(PathBuf::from(
            "test-storage/backup_and_restore_different_coin_type/1.stronghold",
        ))?;

    stronghold.store_mnemonic("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string()).await.unwrap();

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(stronghold))
        .with_client_options(client_options.clone())
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/backup_and_restore_different_coin_type/1")
        .finish()
        .await?;

    // Create one account
    manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    manager
        .backup(
            PathBuf::from("test-storage/backup_and_restore_different_coin_type/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;

    // restore from backup

    let stronghold = StrongholdSecretManager::builder().build(PathBuf::from(
        "test-storage/backup_and_restore_different_coin_type/2.stronghold",
    ))?;

    let restore_manager = AccountManager::builder()
        .with_storage_path("test-storage/backup_and_restore_different_coin_type/2")
        .with_secret_manager(SecretManager::Stronghold(stronghold))
        .with_client_options(ClientOptions::new().with_node(common::NODE_OTHER)?)
        // Build with a different coin type, to check if it gets replaced by the one from the backup
        .with_coin_type(IOTA_COIN_TYPE)
        .finish()
        .await?;

    // restore with ignore_if_coin_type_mismatch: Some(true) to not overwrite the coin type
    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore_different_coin_type/backup.stronghold"),
            stronghold_password.to_string(),
            Some(true),
        )
        .await?;

    // Validate restored data

    // No accounts restored, because the coin type was different
    assert!(restore_manager.get_accounts().await?.is_empty());

    // Restored coin type is not used and it's still the same one
    let new_account = restore_manager.create_account().finish().await?;
    assert_eq!(new_account.read().await.coin_type(), &IOTA_COIN_TYPE);
    // secret manager is the same
    assert_eq!(
        new_account.addresses().await?[0].address().to_bech32(),
        "smr1qrpwecegav7eh0z363ca69laxej64rrt4e3u0rtycyuh0mam3vq3ulygj9p"
    );

    // compare restored client options
    let client_options = restore_manager.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse(common::NODE_OTHER).unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    common::tear_down(storage_path)
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use iota_client::{
    node_manager::node::{Node, NodeDto, Url},
    secret::{mnemonic::MnemonicSecretManager, stronghold::StrongholdSecretManager, SecretManager},
};
use iota_wallet::{account_manager::AccountManager, ClientOptions, Result};

#[tokio::test]
#[cfg(all(feature = "stronghold", feature = "storage"))]
// Backup and restore with Stronghold
async fn backup_and_restore() -> Result<()> {
    std::fs::remove_dir_all("test-storage/backup_and_restore").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://some-not-default-node:14265")?;

    let stronghold_password = "some_hopefully_secure_password";

    // Create directory if not existing, because stronghold panics otherwise
    std::fs::create_dir_all("test-storage/backup_and_restore").unwrap_or(());
    let mut stronghold_secmngr = StrongholdSecretManager::builder()
        .password(stronghold_password)
        .snapshot_path(PathBuf::from("test-storage/backup_and_restore/1.stronghold"))
        .try_build()?;

    stronghold_secmngr.store_mnemonic("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string()).await.unwrap();

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(stronghold_secmngr))
        .with_client_options(client_options.clone())
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

    let stronghold_secmngr = StrongholdSecretManager::builder()
        .snapshot_path(PathBuf::from("test-storage/backup_and_restore/2.stronghold"))
        .try_build()?;

    let restore_manager = AccountManager::builder()
        .with_storage_path("test-storage/backup_and_restore/2")
        .with_secret_manager(SecretManager::Stronghold(stronghold_secmngr))
        .with_client_options(ClientOptions::new().with_node("http://some-other-node:14265")?)
        .finish()
        .await?;

    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;

    // Validate restored data

    // compare restored client options
    let client_options = restore_manager.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse("http://some-not-default-node:14265").unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    // Get account
    let recovered_account = restore_manager.get_account("Alice").await?;
    assert_eq!(
        account.list_addresses().await?,
        recovered_account.list_addresses().await?
    );

    // secret manager is the same
    assert_eq!(
        account.generate_addresses(1, None).await?,
        recovered_account.generate_addresses(1, None).await?
    );
    std::fs::remove_dir_all("test-storage/backup_and_restore").unwrap_or(());
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "stronghold", feature = "storage"))]
// Backup and restore with Stronghold and MnemonicSecretManager
async fn backup_and_restore_mnemonic_secret_manager() -> Result<()> {
    std::fs::remove_dir_all("test-storage/backup_and_restore_mnemonic_secret_manager").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://some-not-default-node:14265")?;

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options.clone())
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
    std::fs::create_dir_all("test-storage/backup_and_restore_mnemonic_secret_manager").unwrap_or(());
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
        .with_client_options(ClientOptions::new().with_node("http://some-other-node:14265")?)
        .finish()
        .await?;

    restore_manager
        .restore_backup(
            PathBuf::from("test-storage/backup_and_restore_mnemonic_secret_manager/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;

    // Validate restored data

    // compare restored client options
    let client_options = restore_manager.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse("http://some-not-default-node:14265").unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    // Get account
    let recovered_account = restore_manager.get_account("Alice").await?;
    assert_eq!(
        account.list_addresses().await?,
        recovered_account.list_addresses().await?
    );

    // secret manager is the same
    assert_eq!(
        account.generate_addresses(1, None).await?,
        recovered_account.generate_addresses(1, None).await?
    );
    std::fs::remove_dir_all("test-storage/backup_and_restore_mnemonic_secret_manager").unwrap_or(());
    Ok(())
}

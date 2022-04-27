// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use iota_client::{
    node_manager::node::{Node, NodeDto, Url},
    secret::stronghold::StrongholdSecretManager,
};
use iota_wallet::{account_manager::AccountManager, secret::SecretManagerType, ClientOptions, Result};

#[tokio::test]
// Backup and restore with Stronghold
async fn backup_and_restore() -> Result<()> {
    std::fs::remove_dir_all("test-storage/backup_and_restore").unwrap_or(());
    let client_options = ClientOptions::new().with_node("http://some-not-default-node:14265")?;

    let stronghold_password = "some_hopefully_secure_password";

    // Create directory if not existing, because stronghold panics otherwise
    std::fs::create_dir("test-storage/backup_and_restore").unwrap_or(());
    let mut stronghold_secmngr = StrongholdSecretManager::builder()
        .password(stronghold_password)
        .snapshot_path(PathBuf::from("test-storage/backup_and_restore/1.stronghold"))
        .build();

    stronghold_secmngr.store_mnemonic("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak".to_string()).await.unwrap();

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManagerType::Stronghold(Box::new(stronghold_secmngr)))
        .with_client_options(client_options)
        .with_storage_path("test-storage/backup_and_restore/1")
        .finish()
        .await?;

    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;
    // Generate an additional address
    let first_account_address = account.generate_addresses(1, None).await?;
    manager
        .backup(
            PathBuf::from("test-storage/backup_and_restore/backup.stronghold"),
            stronghold_password.to_string(),
        )
        .await?;
    drop(manager);

    // restore from backup
    let stronghold_secmngr = StrongholdSecretManager::builder()
        .password(stronghold_password)
        .snapshot_path(PathBuf::from("test-storage/backup_and_restore/2.stronghold"))
        .build();

    let manager2 = AccountManager::builder()
        .with_storage_path("test-storage/backup_and_restore/2")
        .with_secret_manager(SecretManagerType::Stronghold(Box::new(stronghold_secmngr)))
        .with_backup_path(PathBuf::from("test-storage/backup_and_restore/backup.stronghold"))
        .finish()
        .await?;

    // todo: remove when accounts are restored with the backup
    let account2 = manager2
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;
    // Generate an additional address
    let first_account_address2 = account2.generate_addresses(1, None).await?;
    // Adresses are the same
    assert_eq!(first_account_address, first_account_address2);

    // todo enable this when accounts are included in the backup
    // let recovered_account = manager.get_account("Alice").await?;
    // assert_eq!(account.list_addresses().await?, recovered_account.list_addresses().await?);

    // compare restored client options
    let client_options = manager2.get_client_options().await;
    let node_dto = NodeDto::Node(Node::from(Url::parse("http://some-not-default-node:14265").unwrap()));
    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    std::fs::remove_dir_all("test-storage/backup_and_restore").unwrap_or(());
    Ok(())
}
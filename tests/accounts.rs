// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, signing::mnemonic::MnemonicSigner, ClientOptions, Result};

#[tokio::test]
async fn account_ordering() -> Result<()> {
    std::fs::remove_dir_all("test-storage/account_ordering").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path("test-storage/account_ordering")
        .finish()
        .await?;

    for _ in 0..100 {
        let _account = manager.create_account().finish().await?;
    }
    std::fs::remove_dir_all("test-storage/account_ordering").unwrap_or(());
    #[cfg(debug_assertions)]
    manager.verify_integrity().await?;
    Ok(())
}

#[tokio::test]
async fn account_alias_already_exists() -> Result<()> {
    std::fs::remove_dir_all("test-storage/account_alias_already_exists").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path("test-storage/account_alias_already_exists")
        .finish()
        .await?;

    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;
    assert!(&manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await
        .is_err());
    assert!(&manager
        .create_account()
        .with_alias("alice".to_string())
        .finish()
        .await
        .is_err());
    assert!(&manager
        .create_account()
        .with_alias("ALICE".to_string())
        .finish()
        .await
        .is_err());
    // Other alias works
    assert!(&manager
        .create_account()
        .with_alias("Bob".to_string())
        .finish()
        .await
        .is_ok());

    std::fs::remove_dir_all("test-storage/account_alias_already_exists").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn account_rename_alias() -> Result<()> {
    std::fs::remove_dir_all("test-storage/account_rename_alias").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();
    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path("test-storage/account_rename_alias")
        .finish()
        .await?;

    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    assert_eq!(account.alias().await, "Alice".to_string());

    // rename account
    account.set_alias("Bob").await?;

    assert_eq!(account.alias().await, "Bob".to_string());

    std::fs::remove_dir_all("test-storage/account_rename_alias").unwrap_or(());
    Ok(())
}

#[tokio::test]
async fn account_first_address_exists() -> Result<()> {
    std::fs::remove_dir_all("test-storage/account_first_address_exists").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();
    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .with_storage_path("test-storage/account_first_address_exists")
        .finish()
        .await?;

    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    // When the account is generated, the first public address also gets generated and added to it
    assert_eq!(account.list_addresses().await?.len(), 1);
    // First address is a public address
    assert_eq!(account.list_addresses().await?.first().unwrap().internal(), &false);

    std::fs::remove_dir_all("test-storage/account_first_address_exists").unwrap_or(());
    Ok(())
}

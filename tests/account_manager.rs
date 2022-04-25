// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::node_manager::node::{Node, NodeDto, Url};
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
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();
    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
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

    // Recreate AccountManager with a diferent mnemonic
    let secret_manager2 = MnemonicSecretManager::try_from_mnemonic(
        "route hen wink below army inmate object crew vintage gas best space visit say fortune gown few brain emerge umbrella consider spider digital galaxy",
    )?;
    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager2))
        .with_storage_path("test-storage/different_seed")
        .finish()
        .await?;

    // Generating a new account needs to return an error, because the seed from the signer is different
    assert!(manager
        .create_account()
        .with_alias("Bob".to_string())
        .finish()
        .await
        .is_err());

    std::fs::remove_dir_all("test-storage/different_seed").unwrap_or(());
    Ok(())
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Tests for recovering accounts from mnemonic without a backup

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[ignore]
#[tokio::test]
async fn account_recovery_empty() -> Result<()> {
    std::fs::remove_dir_all("test-storage/account_recovery_empty").unwrap_or(());
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
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path("test-storage/account_recovery_empty")
        .finish()
        .await?;

    let accounts = manager.recover_accounts(0, 2, 2, None).await?;

    // accounts should be empty if no account was created before and no account was found with balance
    assert_eq!(0, accounts.len());
    std::fs::remove_dir_all("test-storage/account_recovery_empty").unwrap_or(());
    Ok(())
}

#[ignore]
#[tokio::test]
async fn account_recovery_existing_accounts() -> Result<()> {
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
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // create two accounts
    manager.create_account().finish().await?;
    manager.create_account().finish().await?;

    let accounts = manager.recover_accounts(0, 2, 2, None).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&(index as u32), account.read().await.index());
    }
    // accounts should be 2 because we created 2 accounts before and no new account was found with balance
    assert_eq!(2, accounts.len());
    Ok(())
}

#[ignore]
#[tokio::test]
async fn account_recovery_with_balance() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic with balance on account with index 2 and address key_index 2 on the public address
    // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // create one account
    manager.create_account().finish().await?;

    let accounts = manager.recover_accounts(0, 3, 2, None).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&(index as u32), account.read().await.index());
    }
    // accounts should be 3 because account with index 2 has balance
    assert_eq!(3, accounts.len());

    let account_with_balance = accounts[2].read().await;
    // should have 3 addresses, index 0, 1, 2, all above should be deleted again
    assert_eq!(3, account_with_balance.public_addresses().len());
    Ok(())
}

#[ignore]
#[tokio::test]
async fn account_recovery_with_balance_and_empty_addresses() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic with balance on account with index 2 and address key_index 2 on the public address
    // atoi1qqt9tygh7h7s3l66m242hee6zwp98x90trejt9zya4vcnf5u34yluws9af6
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // create one account
    manager.create_account().finish().await?;
    manager.create_account().finish().await?;
    let account = manager.create_account().finish().await?;
    let _addresses = account.generate_addresses(5, None).await?;

    let accounts = manager.recover_accounts(0, 3, 2, None).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&(index as u32), account.read().await.index());
    }
    // accounts should be 3 because account with index 2 has balance
    assert_eq!(3, accounts.len());

    let account_with_balance = accounts[2].read().await;
    // should have 10 addresses, because we generated 10 before, even thought they don't all have funds
    assert_eq!(5, account_with_balance.public_addresses().len());
    Ok(())
}

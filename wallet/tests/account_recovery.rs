// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Tests for recovering accounts from mnemonic without a backup

mod common;

use std::time::Duration;

use iota_client::{constants::SHIMMER_COIN_TYPE, Client};
use iota_wallet::{
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Result,
};

#[ignore]
#[tokio::test]
async fn account_recovery_empty() -> Result<()> {
    let storage_path = "test-storage/account_recovery_empty";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let accounts = manager.recover_accounts(0, 2, 2, None).await?;

    // accounts should be empty if no account was created before and no account was found with balance
    assert_eq!(0, accounts.len());
    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn account_recovery_existing_accounts() -> Result<()> {
    let storage_path = "test-storage/account_recovery_existing_accounts";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

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
    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn account_recovery_with_balance_and_empty_addresses() -> Result<()> {
    let storage_path = "test-storage/account_recovery_with_balance_and_empty_addresses";
    common::setup(storage_path)?;

    let mnemonic = Client::generate_mnemonic()?;
    let client = Client::builder().with_node(common::NODE_LOCAL)?.finish()?;

    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(&mnemonic)?);

    let address = client
        .get_addresses(&secret_manager)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_bech32_hrp(client.get_bech32_hrp().await?)
        .with_account_index(2)
        .with_range(2..3)
        .finish()
        .await?;

    // Add funds to the address with account index 2 and address key_index 2, so recover works
    iota_client::request_funds_from_faucet(common::FAUCET_URL, &address[0]).await?;

    // Wait for faucet transaction
    tokio::time::sleep(Duration::new(10, 0)).await;

    let manager = common::make_manager(storage_path, Some(&mnemonic), None).await?;

    let accounts = manager.recover_accounts(0, 3, 2, None).await?;

    // accounts should still be ordered
    for (index, account) in accounts.iter().enumerate() {
        assert_eq!(&(index as u32), account.read().await.index());
    }
    // accounts should be 3 because account with index 2 has balance
    assert_eq!(3, accounts.len());

    let account_with_balance = accounts[2].read().await;
    // should have 3 addresses
    assert_eq!(3, account_with_balance.public_addresses().len());
    common::tear_down(storage_path)
}

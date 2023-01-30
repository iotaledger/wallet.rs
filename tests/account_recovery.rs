// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

// Tests for recovering accounts from mnemonic without a backup
use std::time::Duration;

use iota_wallet::Result;

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

    // Add funds to the address so recover works
    iota_client::request_funds_from_faucet(
        "http://localhost:8091/api/enqueue",
        "rms1qryc7d9nlv4q4jpds04nkld2p6qm7xyhz2rcvg5w98jgn2kxjqw5xwrl0lm",
    )
    .await?;

    // Wait for faucet transaction
    tokio::time::sleep(Duration::new(5, 0)).await;

    // mnemonic with balance on account with index 2 and address key_index 2 on the public address
    // rms1qryc7d9nlv4q4jpds04nkld2p6qm7xyhz2rcvg5w98jgn2kxjqw5xwrl0lm
    let mnemonic = "merit blame slam front add unknown winner wait matrix carbon lion cram picnic mushroom turn stadium bright wheel open tragic liar will law time";
    let manager = common::make_manager(storage_path, Some(mnemonic), None).await?;

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

// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use iota_client::block::output::{
    unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition},
    BasicOutputBuilder, UnlockCondition,
};
use iota_wallet::Result;

#[ignore]
#[tokio::test]
async fn balance_expiration() -> Result<()> {
    let storage_path = "test-storage/balance_expiration";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account_0 = &common::create_accounts_with_funds(&manager, 1).await?[0];
    let account_1 = manager.create_account().finish().await?;
    let account_2 = manager.create_account().finish().await?;

    let seconds_until_expired = 20;
    let token_supply = account_0.client().get_token_supply().await?;
    let outputs = vec![
        BasicOutputBuilder::new_with_amount(1_000_000)?
            // Send to account 1 with expiration to account 2, both have no amount yet
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(
                    *account_1.addresses().await?[0].address().as_ref(),
                )),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    *account_2.addresses().await?[0].address().as_ref(),
                    // Current time + 20s
                    account_0.client().get_time_checked().await? + seconds_until_expired,
                )?),
            ])
            .finish_output(token_supply)?,
    ];

    let balance_before_tx = account_0.balance().await?;
    let tx = account_0.send(outputs, None).await?;
    let balance_after_tx = account_0.balance().await?;
    // Total doesn't change before syncing after tx got confirmed
    assert_eq!(balance_before_tx.base_coin.total, balance_after_tx.base_coin.total);
    assert_eq!(balance_after_tx.base_coin.available, 0);

    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Account 1 balance before expiration
    let balance = account_1.sync(None).await?;
    assert_eq!(balance.potentially_locked_outputs.len(), 1);
    assert_eq!(balance.base_coin.total, 0);
    assert_eq!(balance.base_coin.available, 0);

    // Account 2 balance before expiration
    let balance = account_2.sync(None).await?;
    assert_eq!(balance.potentially_locked_outputs.len(), 1);
    assert_eq!(balance.base_coin.total, 0);
    assert_eq!(balance.base_coin.available, 0);

    // Wait until expired
    tokio::time::sleep(std::time::Duration::from_secs(seconds_until_expired.into())).await;

    // Account 1 balance after expiration
    let balance = account_1.sync(None).await?;
    assert_eq!(balance.potentially_locked_outputs.len(), 0);
    assert_eq!(balance.base_coin.total, 0);
    assert_eq!(balance.base_coin.available, 0);

    // Account 2 balance after expiration
    let balance = account_2.sync(None).await?;
    assert_eq!(balance.potentially_locked_outputs.len(), 0);
    assert_eq!(balance.base_coin.total, 1_000_000);
    assert_eq!(balance.base_coin.available, 1_000_000);

    common::tear_down(storage_path)
}

// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use iota_wallet::{AddressWithAmount, Result};

#[ignore]
#[tokio::test]
async fn consolidation() -> Result<()> {
    let storage_path = "test-storage/consolidation";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account_0 = &common::create_accounts_with_funds(&manager, 1).await?[0];
    let account_1 = manager.create_account().finish().await?;

    // Send 10 outputs to account_1
    let amount = 1_000_000;
    let tx = account_0
        .send_amount(
            vec![
                AddressWithAmount {
                    address: account_1.addresses().await?[0].address().to_bech32(),
                    amount,
                };
                10
            ],
            None,
        )
        .await?;

    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = account_1.sync(None).await.unwrap();
    assert_eq!(balance.base_coin.available, 10 * amount);
    assert_eq!(account_1.unspent_outputs(None).await?.len(), 10);

    let tx = account_1.consolidate_outputs(true, None).await?;
    account_1
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    let balance = account_1.sync(None).await.unwrap();
    // Balance still the same
    assert_eq!(balance.base_coin.available, 10 * amount);
    // Only one unspent output
    assert_eq!(account_1.unspent_outputs(None).await?.len(), 1);

    common::tear_down(storage_path)
}

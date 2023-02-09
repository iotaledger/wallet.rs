// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use iota_client::block::output::{
    unlock_condition::{
        AddressUnlockCondition, ExpirationUnlockCondition, GovernorAddressUnlockCondition,
        StateControllerAddressUnlockCondition, StorageDepositReturnUnlockCondition,
    },
    AliasId, AliasOutputBuilder, BasicOutputBuilder, NftId, NftOutputBuilder, UnlockCondition,
};
use iota_wallet::{account::SyncOptions, Result};

#[ignore]
#[tokio::test]
async fn sync_only_most_basic_outputs() -> Result<()> {
    let storage_path = "test-storage/sync_only_most_basic_outputs";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;

    let account_0 = &common::create_accounts_with_funds(&manager, 1).await?[0];
    let account_1 = manager.create_account().finish().await?;

    let account_1_address = *account_1.addresses().await?[0].address().as_ref();

    let token_supply = account_0.client().get_token_supply().await?;
    // Only one basic output without further unlock conditions
    let outputs = vec![
        BasicOutputBuilder::new_with_amount(1_000_000)?
            .with_unlock_conditions(vec![UnlockCondition::Address(AddressUnlockCondition::new(
                account_1_address,
            ))])
            .finish_output(token_supply)?,
        BasicOutputBuilder::new_with_amount(1_000_000)?
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(account_1_address)),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    account_1_address,
                    // Already expired
                    account_0.client().get_time_checked().await? - 5000,
                )?),
            ])
            .finish_output(token_supply)?,
        BasicOutputBuilder::new_with_amount(1_000_000)?
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(account_1_address)),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    account_1_address,
                    // Not expired
                    account_0.client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
        BasicOutputBuilder::new_with_amount(1_000_000)?
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(account_1_address)),
                UnlockCondition::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                    account_1_address,
                    1_000_000,
                    token_supply,
                )?),
            ])
            .finish_output(token_supply)?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())?
            .with_unlock_conditions(vec![UnlockCondition::Address(AddressUnlockCondition::new(
                account_1_address,
            ))])
            .finish_output(token_supply)?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())?
            .with_unlock_conditions(vec![
                UnlockCondition::Address(AddressUnlockCondition::new(account_1_address)),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    account_1_address,
                    account_0.client().get_time_checked().await? + 5000,
                )?),
            ])
            .finish_output(token_supply)?,
        AliasOutputBuilder::new_with_amount(1_000_000, AliasId::null())?
            .with_unlock_conditions(vec![
                UnlockCondition::StateControllerAddress(StateControllerAddressUnlockCondition::new(account_1_address)),
                UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(account_1_address)),
            ])
            .finish_output(token_supply)?,
    ];

    let tx = account_0.send(outputs, None).await?;
    account_0
        .retry_transaction_until_included(&tx.transaction_id, None, None)
        .await?;

    // Sync with sync_only_most_basic_outputs: true, only the first output should be synced
    let balance = account_1
        .sync(Some(SyncOptions {
            sync_only_most_basic_outputs: true,
            ..Default::default()
        }))
        .await?;
    assert_eq!(balance.potentially_locked_outputs.len(), 0);
    assert_eq!(balance.nfts.len(), 0);
    assert_eq!(balance.aliases.len(), 0);
    let unspent_outputs = account_1.unspent_outputs(None).await?;
    assert_eq!(unspent_outputs.len(), 1);
    unspent_outputs.into_iter().for_each(|output_data| {
        assert!(output_data.output.is_basic());
        assert_eq!(output_data.output.unlock_conditions().unwrap().len(), 1);
        assert_eq!(
            *output_data
                .output
                .unlock_conditions()
                .unwrap()
                .address()
                .unwrap()
                .address(),
            account_1_address
        );
    });

    common::tear_down(storage_path)
}

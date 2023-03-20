// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example check_unlock_conditions --release
// In this example we check if an output has only an address unlock condition and that the address is from the account.

use iota_client::block::output::{unlock_condition::AddressUnlockCondition, BasicOutputBuilder, UnlockCondition};
use iota_wallet::{account::types::AddressWrapper, account_manager::AccountManager, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let account_addresses: Vec<AddressWrapper> = account
        .addresses()
        .await?
        .into_iter()
        .map(|a| a.address().clone())
        .collect();

    let output = BasicOutputBuilder::new_with_amount(1_000_000)?
        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
            *account_addresses[0].as_ref(),
        )))
        .finish_output(account.client().get_token_supply().await?)?;

    let controlled_by_account = if let [UnlockCondition::Address(address_unlock_condition)] = output
        .unlock_conditions()
        .expect("output needs to have unlock conditions")
        .as_ref()
    {
        // Check that address in the unlock condition belongs to the account
        account_addresses
            .iter()
            .any(|a| a.as_ref() == address_unlock_condition.address())
    } else {
        false
    };

    println!(
        "The output has only an address unlock condition and the address is from the account: {controlled_by_account:?}"
    );

    Ok(())
}

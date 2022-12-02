// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example output_consolidation --release
// In this example we will consolidate basic outputs from an account with only an AddressUnlockCondition by sending them
// to the same address again
// Rename `.env.example` to `.env` first

use std::env;

use dotenv::dotenv;
use iota_client::{
    constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
};
use iota_wallet::{account_manager::AccountManager, ClientOptions, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let mnemonic: &str = &env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap();
    let mnemonic_secret_manager = MnemonicSecretManager::try_from_mnemonic(mnemonic).unwrap();
    let secret_manager = SecretManager::Mnemonic(mnemonic_secret_manager);

    let client_options = ClientOptions::new().with_node(&env::var("NODE_URL").unwrap())?;

    // Create the account manager
    let manager = AccountManager::builder()
        .with_secret_manager(secret_manager)
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Sync account to make sure account is updated with outputs from previous examples
    let _ = account.sync(None).await?;

    // List unspent outputs before consolidation.
    // The output we created with example `03_get_funds` and the basic output from `09_mint_native_tokens` have only one
    // unlock condition and it is an `AddressUnlockCondition`, and so they are valid for consolidation. They have the
    // same `AddressUnlockCondition`(the first address of the account), so they will be consolidated into one
    // output.
    let outputs = account.unspent_outputs(None).await?;
    println!("Outputs before consolidation:");
    outputs.iter().for_each(|output_data| {
        println!(
            "address: {:?}\n amount: {:?}\n native tokens: {:?}\n",
            output_data.address.to_bech32("rms"),
            output_data.output.amount(),
            output_data.output.native_tokens()
        )
    });

    // Consolidate unspent outputs and print the consolidation transaction IDs
    // Set `force` to true to force the consolidation even though the `output_consolidation_threshold` isn't reached
    let transaction = account.consolidate_outputs(true, None).await?;
    println!("Consolidation transaction id:\n{:?}\n", transaction);

    // Wait for the consolidation transaction to get confirmed
    account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;

    // Sync account
    let _ = account.sync(None).await?;

    // Outputs after consolidation
    let outputs = account.unspent_outputs(None).await?;
    println!("Outputs after consolidation:");
    outputs.iter().for_each(|output_data| {
        println!(
            "address: {:?}\n amount: {:?}\n native tokens: {:?}\n",
            output_data.address.to_bech32("rms"),
            output_data.output.amount(),
            output_data.output.native_tokens()
        )
    });

    Ok(())
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example split_funds --release

use std::{env, time::Instant};

use dotenv::dotenv;
use iota_client::bee_block::output::{
    unlock_condition::{AddressUnlockCondition, UnlockCondition},
    BasicOutputBuilder,
};
use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let client_options = ClientOptions::new()
        .with_node(&env::var("NODE_URL").unwrap())?
        .with_node_sync_disabled();

    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap())?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "logger";
    let account = match manager.get_account(account_alias.to_string()).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager
                .create_account()
                .with_alias(account_alias.to_string())
                .finish()
                .await?
        }
    };

    let _address = account.generate_addresses(5, None).await?;
    let addresses = account.generate_addresses(300, None).await?;
    let mut bech32_addresses = Vec::new();
    for ad in addresses {
        bech32_addresses.push(ad.address().to_bech32());
    }

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    let addresses_with_unspent_outputs = account.list_addresses_with_unspent_outputs().await?;
    println!("Addresses with balance: {}", addresses_with_unspent_outputs.len());

    // send transaction
    for chunk in addresses.chunks(100).map(|x| x.to_vec()) {
        let outputs = chunk
            .into_iter()
            .map(|a| {
                BasicOutputBuilder::new_with_amount(1_000_000)
                    .unwrap()
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        *a.address().as_ref(),
                    )))
                    .finish_output()
                    .unwrap()
            })
            .collect();
        match account.send(outputs, None).await {
            Ok(tx) => println!(
                "Block sent: {}/api/core/v2/blocks/{}",
                &env::var("NODE_URL").unwrap(),
                tx.block_id.expect("No block created yet")
            ),
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}

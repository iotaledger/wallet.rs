// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example threads --release

// In this example we will spam transactions from multiple threads simultaneously to our own address

use std::env;

use iota_client::{
    bee_block::output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder,
    },
    constants::SHIMMER_COIN_TYPE,
};
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv::dotenv().ok();

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
    let account_alias = "thread_account";
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

    // One address gets generated during account creation
    let address = account.list_addresses().await?[0].address().clone();
    println!("{}", address.to_bech32());

    let balance = account.sync(None).await?;
    println!("Balance: {:?}", balance);

    if balance.base_coin.available == 0 {
        panic!("Account has no available balance");
    }

    for _ in 0..1000 {
        let mut threads = Vec::new();
        for n in 0..10 {
            let account_ = account.clone();
            let address_ = *address.as_ref();

            threads.push(async move {
                tokio::spawn(async move {
                    // send transaction
                    let outputs = vec![
                        BasicOutputBuilder::new_with_amount(1_000_000)?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address_)))
                            .finish_output()?;
                        // amount of outputs in the transaction (one additional output might be added for the remaining amount)
                        1
                    ];
                    let tx = account_.send(outputs, None).await?;
                    if let Some(block_id) = tx.block_id {
                        println!(
                            "Block from thread {} sent: {}/api/core/v2/blocks/{}",
                            n,
                            &env::var("NODE_URL").unwrap(),
                            block_id
                        );
                    }
                    iota_wallet::Result::Ok(n)
                })
                .await
            });
        }

        let results = futures::future::try_join_all(threads).await?;
        for thread in results {
            if let Err(e) = thread {
                println!("{e}");
                // Sync when getting an error, because that's probably when no outputs are available anymore
                println!("Syncing account...");
                account.sync(None).await?;
            }
        }
    }
    Ok(())
}

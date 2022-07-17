// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 15_destroy_alias --release
// In this example we will destroy an existing alias output. This is only possible if possible foundry outputs have
// circulating supply of 0. Rename `.env.example` to `.env` first

use std::{env, str::FromStr};

use iota_client::block::output::AliasId;
use iota_wallet::{account_manager::AccountManager, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv::dotenv().ok();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    let balance = account.balance().await?;
    println!("Balance before destroying:\n{balance:?}",);

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Replace with an AliasId that is available in the account
    let alias_id = AliasId::from_str("0x57f1bafae0ef43190597a0dfe72ef1477b769560203c1854c6fb427c486e6530")?;
    let transaction = account.destroy_alias(alias_id, None).await?;

    let _ = match transaction.block_id {
        Some(block_id) => account.retry_until_included(&block_id, None, None).await?,
        None => {
            return Err(iota_wallet::Error::BurningOrMeltingFailed(
                "Burn nft failed to submitted".to_string(),
            ));
        }
    };

    let balance = account.sync(None).await?;

    println!("Balance after destroying:\n{balance:?}",);

    Ok(())
}

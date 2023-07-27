// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --release --all-features --example migrate_stronghold_snapshot_v2_to_v3

use iota_wallet::{account_manager::AccountManager, Error};

const STORAGE_PATH: &str = "./storage";
const V2_PATH: &str = "./storage/wallet.stronghold";
const V3_PATH: &str = "./storage/wallet.stronghold";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let manager = AccountManager::builder()
        .with_storage(STORAGE_PATH, None)?
        .with_skip_polling()
        .finish()
        .await?;

    // This should fail with error, migration required.
    let error = if let Err(e) = manager.set_stronghold_password("password").await {
        e
    } else {
        panic!("should be an error");
    };
    println!("Creating a stronghold failed with error: {error:?}");

    println!("Migrating stronghold snapshot from v2 to v3");
    AccountManager::migrate_stronghold_snapshot_v2_to_v3(
        V2_PATH,
        "current_password",
        "wallet.rs",
        100,
        Some(V3_PATH),
        Some("new_password"),
    )
    .unwrap();

    // This shouldn't fail anymore as snapshot has been migrated.
    manager.set_stronghold_password("new_password").await?;

    // Generate addresses with custom account index and range
    // let addresses = GetAddressesBuilder::new(&SecretManager::Stronghold(stronghold_secret_manager))
    //     .with_bech32_hrp(SHIMMER_TESTNET_BECH32_HRP)
    //     .with_coin_type(SHIMMER_COIN_TYPE)
    //     .with_account_index(0)
    //     .with_range(0..1)
    //     .finish()
    //     .await?;

    // println!("First public address: {}", addresses[0]);

    Ok(())
}

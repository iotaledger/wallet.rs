// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    account_manager::{AccountManager, MigrationDataFinder},
    client::ClientOptionsBuilder,
    signing::SignerType,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let manager = AccountManager::builder().finish().await.unwrap();
    manager.set_stronghold_password("password").await.unwrap();
    manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .build()
        .unwrap();
    let _account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    // Migration
    let legacy_node = "https://nodes.devnet.iota.org";
    let seed = "TRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEED";
    let min_weight_magnitude = 9;

    // Get account data
    let mut address_index = 0;
    let gap_limit = 30;
    let yes = vec!['Y', 'y'];
    let mut user_input = String::new();
    let mut migration_data = None;
    while !yes.contains(&user_input.chars().next().unwrap_or('N')) {
        let account_migration_data = manager
            .get_migration_data(
                MigrationDataFinder::new(&[legacy_node], seed)?
                    .with_initial_address_index(address_index)
                    .with_gap_limit(gap_limit)
                    .with_permanode("https://chronicle.iota.org/api"),
            )
            .await?;
        println!(
            "Is {}i the correct balance? Type Y to continue or N to search for more balance",
            account_migration_data.balance
        );
        user_input = String::new();
        std::io::stdin().read_line(&mut user_input).unwrap();
        address_index += gap_limit;
        migration_data = Some(account_migration_data);
    }

    let migration_data = migration_data.unwrap();

    let mut bundle_hashes = Vec::new();
    for input in migration_data.inputs.into_iter() {
        let bundle = manager
            .create_migration_bundle(
                seed,
                &[input.index],
                true,
                Duration::from_secs(40),
                0,
                "./migration.log",
            )
            .await?;
        bundle_hashes.push(bundle.bundle_hash().to_string());
    }

    for hash in bundle_hashes {
        manager
            .send_migration_bundle(&[legacy_node], &hash, min_weight_magnitude)
            .await?;
        println!("Bundle sent, hash: {}", hash);
    }

    Ok(())
}

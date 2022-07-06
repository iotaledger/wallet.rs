// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example ledger_nano --release --features=ledger_nano

use std::time::Instant;

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{ledger_nano::LedgerSecretManager, SecretManager},
    AddressWithAmount, ClientOptions, Result,
};

// In this example we will create addresses with a ledger nano hardware wallet
// To use the ledger nano simulator clone https://github.com/iotaledger/ledger-shimmer-app, run `git submodule init && git submodule update --recursive`,
// then `./build.sh -m nanos|nanox|nanosplus -s` and use `true` in `LedgerSecretManager::new(true)`.

#[tokio::main]
async fn main() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let secret_manager = LedgerSecretManager::new(true);

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::LedgerNano(secret_manager))
        .with_storage_path("ledger_nano_walletdb")
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    println!("{:?}", manager.get_ledger_status().await?);

    // Get account or create a new one
    let account_alias = "ledger";
    let account = match manager.get_account(account_alias).await {
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

    let address = account.generate_addresses(1, None).await?;

    println!("{:?}", address);

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    // send transaction
    let outputs = vec![AddressWithAmount {
        address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
        amount: 1_000_000,
    }];
    let tx = account.send_amount(outputs, None).await?;

    println!(
        "Transaction: {} Block sent: http://localhost:14265/api/v2/blocks/{}",
        tx.transaction_id,
        tx.block_id.expect("No block created yet")
    );

    Ok(())
}

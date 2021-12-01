// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example wallet --release

use std::time::Instant;
use wallet_core::{
    account::{types::OutputKind, RemainderValueStrategy, TransferOptions, TransferOutput},
    account_manager::AccountManager,
    client::options::ClientOptionsBuilder,
    logger::{init_logger, LevelFilter},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptionsBuilder::new()
        .with_node("https://comnet.tanglebay.com")?
        // .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        // .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        // .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        // .with_node("https://chrysalis-nodes.iota.org/")?
        // .with_node("http://localhost:14265")?
        .with_node_sync_disabled()
        .finish()
        .unwrap();

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .finish()
        .await?;
    // manager.set_stronghold_password("password").await?;

    // Get account or create a new one
    let account_alias = "logger";
    let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;
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

    // let accounts = manager.get_accounts().await?;
    // println!("Accounts: {:?}", accounts);

    let _address = account.generate_addresses(5, None).await?;

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    let addresses_with_balance = account.list_addresses_with_balance().await?;
    println!("Addresses with balance: {}", addresses_with_balance.len());

    // send transaction
    let outputs = vec![TransferOutput {
        address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
        amount: 1_000_000,
        // we create a dust allowance outputs so we can reuse our address even with remainder
        output_kind: Some(OutputKind::SignatureLockedDustAllowance),
    }];
    // let res = account.send(outputs, None).await?;
    let res = account
        .send(
            outputs,
            Some(TransferOptions {
                remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
                ..Default::default()
            }),
        )
        .await?;
    println!(
        "Transaction: {} Message sent: https://explorer.iota.org/devnet/message/{}",
        res.transaction_id,
        res.message_id.expect("No message created yet")
    );
    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    // // switch to mainnet
    // let client_options = ClientOptionsBuilder::new()
    //     .with_node("https://chrysalis-nodes.iota.org/")?
    //     .with_node("https://chrysalis-nodes.iota.cafe/")?
    //     .with_node_sync_disabled()
    //     .finish()
    //     .unwrap();
    // let now = Instant::now();
    // manager.set_client_options(client_options).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", account.balance().await?);

    // // switch back to testnet
    // let client_options = ClientOptionsBuilder::new()
    //     .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
    //     .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
    //     .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
    //     .with_node_sync_disabled()
    //     .finish()
    //     .unwrap();
    // let now = Instant::now();
    // manager.set_client_options(client_options).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", account.balance().await?);

    Ok(())
}

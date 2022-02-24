// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example wallet --release

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder, Output,
    },
};
use iota_wallet::{
    account::{RemainderValueStrategy, TransferOptions},
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    ClientOptions, Result,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil")?;

    let manager = AccountManager::builder()
        .with_signer(signer)
        .with_client_options(client_options)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "logger";
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
    let outputs = vec![Output::Basic(
        BasicOutputBuilder::new(1_000_000)?
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                Address::try_from_bech32("atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e")?,
            )))
            .finish()?,
    )];
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
        "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
        res.transaction_id,
        res.message_id.expect("No message created yet")
    );
    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    // // switch to mainnet
    // let client_options = ClientOptions::new()
    //     .with_node("https://chrysalis-nodes.iota.org/")?
    //     .with_node_sync_disabled();
    // let now = Instant::now();
    // manager.set_client_options(client_options).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", account.balance().await?);

    // // switch back to testnet
    // let client_options = ClientOptions::new()
    //     .with_node("http://localhost:14265")?
    //     .with_node_sync_disabled();
    // let now = Instant::now();
    // manager.set_client_options(client_options).await?;
    // println!("Syncing took: {:.2?}", now.elapsed());
    // println!("Balance: {:?}", account.balance().await?);

    Ok(())
}

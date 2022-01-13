// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example split_funds --release

use iota_wallet::{
    account::{types::OutputKind, RemainderValueStrategy, TransferOptions, TransferOutput},
    account_manager::AccountManager,
    client::options::ClientOptionsBuilder,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    Result,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptionsBuilder::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled()
        .finish()?;

    let signer = MnemonicSigner::new("giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally")?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_signer(signer)
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
    let _address = account.generate_addresses(300, None).await?;
    let mut bech32_addresses = Vec::new();
    for ad in _address {
        bech32_addresses.push(ad.address().to_bech32());
    }

    let addresses = account.list_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    let addresses_with_balance = account.list_addresses_with_balance().await?;
    println!("Addresses with balance: {}", addresses_with_balance.len());

    // send transaction
    for chunk in bech32_addresses.chunks(100).map(|x| x.to_vec()).into_iter() {
        let outputs = chunk
            .into_iter()
            .map(|a| TransferOutput {
                address: a.to_string(),
                amount: 1_000_000,
                output_kind: Some(OutputKind::Extended),
            })
            .collect();
        match account
            .send(
                outputs,
                Some(TransferOptions {
                    remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
                    ..Default::default()
                }),
            )
            .await
        {
            Ok(res) => println!(
                "Message sent: https://explorer.iota.org/devnet/message/{}",
                res.message_id.expect("No message created yet")
            ),
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}

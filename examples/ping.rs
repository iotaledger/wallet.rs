// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example ping --release

// In this example we will try to send transactions from multiple threads simultaneously to the first 1000 addresses of
// the second account (pong_account)

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
    init_logger("ping-wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe")?
        // .with_node("https://chrysalis-nodes.iota.org/")?
        // .with_node("http://localhost:14265")?
        .with_node_sync_disabled()
        .finish()
        .unwrap();

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_storage_folder("pingdb")
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "ping";
    let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;
    let ping_account = match manager.get_account(account_alias.to_string()).await {
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
    let account_alias = "pong";
    let pong_account = match manager.get_account(account_alias.to_string()).await {
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

    let amount_addresses = 100;
    // generate addresses so we find all funds
    if ping_account.list_addresses().await?.len() < amount_addresses {
        ping_account.generate_addresses(amount_addresses, None).await?;
    }
    let balance = ping_account.sync(None).await?;
    println!("Balance: {:?}", balance);
    // generate addresses from the second account to which we will send funds
    let pong_addresses = {
        let mut addresses = pong_account.list_addresses().await?;
        if addresses.len() < amount_addresses {
            addresses = pong_account.generate_addresses(amount_addresses, None).await?
        };
        addresses
    };

    for address_index in 0..amount_addresses {
        let mut threads = Vec::new();
        for n in 1..4 {
            let ping_account_ = ping_account.clone();
            let pong_addresses_ = pong_addresses.clone();
            threads.push(async move {
                tokio::spawn(async move {
                    // send transaction
                    let outputs = vec![TransferOutput {
                        address: pong_addresses_[address_index].address().to_bech32(),
                        // send one or two Mi for more different transactions
                        amount: n * 1_000_000,
                        // we create a dust allowance outputs so we can reuse the address even with remainder
                        // output_kind: Some(OutputKind::SignatureLockedSingle),
                        output_kind: Some(OutputKind::SignatureLockedDustAllowance),
                    }];
                    let res = ping_account_
                        .send(
                            outputs,
                            Some(TransferOptions {
                                remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
                                ..Default::default()
                            }),
                        )
                        .await?;
                    println!(
                        "Message from thread {} sent: https://explorer.iota.org/devnet/message/{}",
                        n,
                        res.message_id.expect("No message created yet")
                    );
                    wallet_core::Result::Ok(n)
                })
                .await
            });
        }

        let results = futures::future::try_join_all(threads).await?;
        for thread in results {
            match thread {
                Ok(res) => println!("{}", res),
                Err(e) => println!("{}", e),
            }
        }
        println!("sleep");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    // wait until user press enter so background tasks keep running
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}

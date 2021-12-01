// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example threads --release

// In this example we will try to send transactions from multiple threads simultaneously

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
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "thread_account";
    let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally".to_string();
    manager.store_mnemonic(Some(mnemonic)).await?;
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

    let _address = account.generate_addresses(10, None).await?;
    for ad in _address {
        println!("{}", ad.address().to_bech32());
    }
    let balance = account.sync(None).await?;
    println!("Balance: {:?}", balance);

    for _ in 0..1000 {
        let mut threads = Vec::new();
        for n in 0..5 {
            let account_ = account.clone();
            threads.push(async move {
                tokio::spawn(async move {
                    // send transaction
                    let outputs = vec![TransferOutput {
                        address: "atoi1qz8wq4ln6sn68hvgwp9r26dw3emdlg7at0mrtmhz709zwwcxvpp46xx2cmj".to_string(),
                        amount: 1_000_000,
                        // we create a dust allowance outputs so we can reuse the address even with remainder
                        // output_kind: Some(OutputKind::SignatureLockedSingle),
                        output_kind: Some(OutputKind::SignatureLockedDustAllowance),
                    }];
                    let res = account_
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

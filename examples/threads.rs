// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example threads --release

// In this example we will try to send transactions from multiple threads simultaneously

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder, Output,
    },
};
use iota_wallet::{
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    // init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil")?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_signer(signer)
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
                    let outputs = vec![Output::Basic(
                        BasicOutputBuilder::new(1_000_000)?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                Address::try_from_bech32(
                                    "atoi1qz8wq4ln6sn68hvgwp9r26dw3emdlg7at0mrtmhz709zwwcxvpp46xx2cmj",
                                )?,
                            )))
                            .finish()?,
                    )];
                    let res = account_.send(outputs, None).await?;
                    println!(
                        "Message from thread {} sent: http://localhost:14265/api/v2/messages/{}",
                        n,
                        res.message_id.expect("No message created yet")
                    );
                    iota_wallet::Result::Ok(n)
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

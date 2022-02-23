// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example ping --release

// In this example we will try to send transactions from multiple threads simultaneously to the first 1000 addresses of
// the second account (pong_account)

use iota_client::{
    bee_message::output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder, Output,
    },
    request_funds_from_faucet,
};
use iota_wallet::{
    account::{RemainderValueStrategy, TransferOptions},
    account_manager::AccountManager,
    client::ClientOptions,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    init_logger("ping-wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptions::new()
        .with_node("https://api.stardust-testnet.iotaledger.net")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally")?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_storage_folder("pingdb")
        .with_signer(signer)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "ping";
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

    let amount_addresses = 5;
    // generate addresses so we find all funds
    if ping_account.list_addresses().await?.len() < amount_addresses {
        ping_account
            .generate_addresses(
                (amount_addresses - ping_account.list_addresses().await?.len()) as u32,
                None,
            )
            .await?;
    }
    let balance = ping_account.sync(None).await?;
    println!("Balance: {:?}", balance);
    // generate addresses from the second account to which we will send funds
    let pong_addresses = {
        let mut addresses = pong_account.list_addresses().await?;
        if addresses.len() < amount_addresses {
            addresses = pong_account
                .generate_addresses((amount_addresses - addresses.len()) as u32, None)
                .await?
        };
        println!(
            "{}",
            request_funds_from_faucet(
                "https://faucet.stardust-testnet.iotaledger.net/api/plugins/faucet/v1/enqueue",
                &addresses[0].address().to_bech32()
            )
            .await?
        );
        addresses
    };

    for address_index in 0..1000 {
        let mut threads = Vec::new();
        for n in 1..4 {
            let ping_account_ = ping_account.clone();
            let pong_addresses_ = pong_addresses.clone();
            threads.push(async move {
                tokio::spawn(async move {
                    // send transaction
                    let outputs = vec![Output::Basic(
                        // send one or two Mi for more different transactions
                        BasicOutputBuilder::new(n * 1_000_000)?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                *pong_addresses_[address_index % amount_addresses].address().as_ref(),
                            )))
                            .finish()?,
                    )];
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
            if let Err(e) = thread {
                println!("{}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    // wait until user press enter so background tasks keep running
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(())
}

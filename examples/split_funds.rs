// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example split_funds --release

use iota_client::bee_message::output::{
    unlock_condition::{AddressUnlockCondition, UnlockCondition},
    BasicOutputBuilder, Output,
};
use iota_wallet::{account_manager::AccountManager, signing::mnemonic::MnemonicSigner, ClientOptions, Result};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil")?;

    let manager = AccountManager::builder(signer)
        .with_client_options(client_options)
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
    let addresses = account.generate_addresses(300, None).await?;
    let mut bech32_addresses = Vec::new();
    for ad in addresses {
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
    for chunk in addresses.chunks(100).map(|x| x.to_vec()).into_iter() {
        let outputs = chunk
            .into_iter()
            .map(|a| {
                Output::Basic(
                    BasicOutputBuilder::new(1_000_000)
                        .unwrap()
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                            *a.address().as_ref(),
                        )))
                        .finish()
                        .unwrap(),
                )
            })
            .collect();
        match account.send(outputs, None).await {
            Ok(res) => println!(
                "Message sent: http://localhost:14265/api/v2/messages/{}",
                res.message_id.expect("No message created yet")
            ),
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}

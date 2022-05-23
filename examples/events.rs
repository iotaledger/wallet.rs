// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example events --features=events --release

use iota_client::bee_block::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder,
    },
};
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .finish()
        .await?;

    manager
        .listen(vec![], move |event| {
            println!("Received an event {:?}", event);
        })
        .await;

    // Get account or create a new one
    let account_alias = "event_account";
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

    let balance = account.sync(None).await?;
    println!("Balance: {:?}", balance);

    // send transaction
    let outputs = vec![BasicOutputBuilder::new_with_amount(1_000_000)?
        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
            Address::try_from_bech32("atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e")?.1,
        )))
        .finish_output()?];
    // let res = account.send(outputs, None).await?;
    let res = account.send(outputs, None).await?;
    println!(
        "Transaction: {} Block sent: http://localhost:14265/api/v2/blocks/{}",
        res.transaction_id,
        res.block_id.expect("No block created yet")
    );

    Ok(())
}

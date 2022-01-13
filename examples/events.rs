// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example events --release

use iota_wallet::{
    account::{types::OutputKind, RemainderValueStrategy, TransferOptions, TransferOutput},
    account_manager::AccountManager,
    client::options::ClientOptionsBuilder,
    signing::mnemonic::MnemonicSigner,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
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
    let outputs = vec![TransferOutput {
        address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
        amount: 1_000_000,
        // we create a dust allowance outputs so we can reuse our address even with remainder
        output_kind: Some(OutputKind::Extended),
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

    Ok(())
}

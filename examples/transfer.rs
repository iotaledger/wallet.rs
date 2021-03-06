// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, message::Transfer, signing::SignerType,
};
use std::num::NonZeroU64;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::builder().finish().await.unwrap();
    manager.set_stronghold_password("password").await.unwrap();
    manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.testnet.chrysalis2.com")?
        .build()
        .unwrap();
    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    let address = account.generate_address().await?;
    println!(
        "Send iotas from the faucet to {} and press enter after the transaction got confirmed",
        address.address().to_bech32()
    );
    let mut message = String::new();
    std::io::stdin().read_line(&mut message).unwrap();
    println!("Sending transfer...");
    let message = account
        .transfer(
            Transfer::builder(
                account.latest_address().await.address().clone(),
                NonZeroU64::new(1500000).unwrap(),
            )
            .finish(),
        )
        .await?;
    println!("Message sent: {}", message.id());

    Ok(())
}

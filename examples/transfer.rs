// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example transfer --release

use iota_wallet::{
    account_manager::AccountManager,
    address::{parse, OutputKind},
    client::ClientOptionsBuilder,
    message::Transfer,
    signing::SignerType,
};
use std::num::NonZeroU64;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let manager = AccountManager::builder().finish().await.unwrap();
    manager.set_stronghold_password("password").await.unwrap();

    // Get account or create a new one
    let account_alias = "alias";
    let account = match manager.get_account(account_alias).await {
        Ok(account) => account,
        _ => {
            // first we'll create an example account and store it
            manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
                .build()
                .unwrap();
            manager
                .create_account(client_options)?
                .alias(account_alias)
                .initialise()
                .await?
        }
    };

    let address = account.generate_address().await?;
    println!(
        "Send iotas from the faucet to {} and press enter after the transaction got confirmed",
        address.address().to_bech32()
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    println!("Sending transfer...");
    let message = account
        .transfer(
            Transfer::builder(
                parse("atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r")?,
                NonZeroU64::new(10000000).unwrap(),
                Some(OutputKind::SignatureLockedDustAllowance),
            )
            .finish(),
        )
        .await?;
    println!("Message sent: {}", message.id());

    Ok(())
}

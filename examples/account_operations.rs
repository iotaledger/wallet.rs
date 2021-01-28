// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, message::MessageType, signing::SignerType,
};

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

    // update alias
    account.set_alias("the new alias").await?;
    // get unspent addresses
    let _ = account.list_unspent_addresses();
    // get spent addresses
    let _ = account.list_spent_addresses();
    // get all addresses
    let _ = account.addresses();

    // generate a new unused address
    let _ = account.generate_address().await?;

    // list messages
    let _ = account.list_messages(5, 0, Some(MessageType::Failed));

    Ok(())
}

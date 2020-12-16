// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder, message::Transfer};

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::new().unwrap();
    manager.set_stronghold_password("password").await.unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    let account = manager
        .create_account(client_options)
        .alias("alias")
        .initialise()
        .await?;

    // we need to synchronize with the Tangle first
    let sync_accounts = manager.sync_accounts().await?;
    let sync_account = sync_accounts.first().unwrap();

    sync_account
        .transfer(Transfer::new(
            account.latest_address().await.unwrap().address().clone(),
            150,
        ))
        .await?;

    Ok(())
}

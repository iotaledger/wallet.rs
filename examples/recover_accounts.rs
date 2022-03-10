// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example recover_accounts --release

use iota_wallet::{
    account_manager::AccountManager,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    ClientOptions, Result,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    // init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("flame fever pig forward exact dash body idea link scrub tennis minute surge unaware prosper over waste kitten ceiling human knife arch situate civil")?;

    let manager = AccountManager::builder(signer)
        .with_client_options(client_options)
        .finish()
        .await?;

    let accounts = manager.recover_accounts(2, 2).await?;
    // let accounts = manager.recover_accounts(2, 2).await?;

    for account in accounts.iter() {
        println!("{}", account.read().await.index());
    }
    println!("Accounts len: {:?}", accounts.len());

    // get latest account
    let account = &accounts[accounts.len() - 1];
    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("Syncing took: {:.2?}", now.elapsed());
    println!("Balance: {:?}", balance);

    Ok(())
}

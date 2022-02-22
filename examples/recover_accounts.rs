// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example recover_accounts --release

use iota_wallet::{
<<<<<<< HEAD
    account_manager::AccountManager, client::options::ClientOptionsBuilder, signing::mnemonic::MnemonicSigner, Result,
=======
    account_manager::AccountManager,
    client::ClientBuilder,
    logger::{init_logger, LevelFilter},
    signing::mnemonic::MnemonicSigner,
    Result,
>>>>>>> shimmer-develop
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    // init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientBuilder::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally")?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_signer(signer)
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

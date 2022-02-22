// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example background_syncing --release

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
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Generates a wallet.log file with logs for debugging
    // init_logger("wallet.log", LevelFilter::Debug)?;

    let client_options = ClientBuilder::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    let signer = MnemonicSigner::new("hollow office master ethics infant review action short vivid fix spatial fresh traffic stand car cradle flower goat voyage output word aisle theme village")?;

    let manager = AccountManager::builder()
        .with_client_options(client_options)
        .with_signer(signer)
        .finish()
        .await?;

    // Get account or create a new one
    let account_alias = "logger";
    let account = match manager.get_account(account_alias).await {
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

    account.generate_addresses(1, None).await?;

    manager.start_background_syncing(None, None).await?;
    sleep(Duration::from_secs(10)).await;
    manager.stop_background_syncing()?;
    manager.start_background_syncing(None, None).await?;
    Ok(())
}

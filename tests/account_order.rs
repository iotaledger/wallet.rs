// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, signing::mnemonic::MnemonicSigner, ClientOptions, Result};

#[tokio::test]
async fn account_ordering() -> Result<()> {
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder(signer)
        .with_client_options(client_options)
        .with_storage_folder("test-storage/account_ordering")
        .finish()
        .await?;

    for _ in 0..100 {
        let _account = manager.create_account().finish().await?;
    }
    std::fs::remove_dir_all("test-storage/account_ordering").unwrap_or(());
    manager.verify_integrity().await?;
    Ok(())
}

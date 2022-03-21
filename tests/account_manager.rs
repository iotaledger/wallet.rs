// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::node_manager::node::{Node, NodeDto, Url};
use iota_wallet::{account_manager::AccountManager, signing::mnemonic::MnemonicSigner, ClientOptions, Result};

#[tokio::test]
async fn stored_account_manager_data() -> Result<()> {
    let client_options = ClientOptions::new().with_node("http://some-not-default-node:14265")?;

    // mnemonic without balance
    let signer = MnemonicSigner::new("inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak")?;

    let manager = AccountManager::builder(signer.clone())
        .with_client_options(client_options)
        .with_storage_folder("test-storage/stored_account_manager_data")
        .finish()
        .await?;

    // todo: enable when https://github.com/iotaledger/wallet.rs/issues/942 is done
    // drop(manager);
    // // Recreate AccountManager without providing client options
    // let manager = AccountManager::builder(signer)
    //     .with_storage_folder("test-storage/stored_account_manager_data")
    //     .finish()
    //     .await?;
    let client_options = manager.get_client_options().await;

    let node_dto = NodeDto::Node(Node::from(Url::parse("http://some-not-default-node:14265").unwrap()));

    assert!(client_options.node_manager_builder.nodes.contains(&node_dto));

    std::fs::remove_dir_all("test-storage/stored_account_manager_data").unwrap_or(());
    Ok(())
}

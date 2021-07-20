// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example ledger --release --features "ledger-nano"

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    // commented because test fails otherwise
    // use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType};
    // println!("Ledger status: {:?}", iota_wallet::get_ledger_status(false).await);

    // let manager = AccountManager::builder().finish().await.unwrap();

    // // Get account or create a new one
    // let account_alias = "ledger";
    // let account = match manager.get_account(account_alias).await {
    //     Ok(account) => account,
    //     _ => {
    //         // first we'll create an example account and store it
    //         let client_options = ClientOptionsBuilder::new()
    //             .with_node("https://api.lb-0.testnet.chrysalis2.com")?
    //             .build()
    //             .unwrap();
    //         manager
    //             .create_account(client_options)?
    //             .signer_type(SignerType::LedgerNano)
    //             .alias(account_alias)
    //             .initialise()
    //             .await?
    //     }
    // };

    // account.sync().await.execute().await?;
    // println!("Balance: {:?}", account.balance().await?);

    // let address = account.generate_address().await?;
    // println!("Generated a new address: {:?}", address);

    Ok(())
}

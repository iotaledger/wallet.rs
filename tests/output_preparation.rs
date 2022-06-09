// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    account::OutputOptions,
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

#[ignore]
#[tokio::test]
async fn output_preparation() -> Result<()> {
    std::fs::remove_dir_all("test-storage/output_preparation").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("https://api.alphanet.iotaledger.net/")?
        .with_node_sync_disabled();

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_storage_path("test-storage/output_preparation")
        .finish()
        .await?;

    let account = manager.create_account().finish().await?;

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 500,
                assets: None,
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 234000);
    // address and sdr unlock condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    std::fs::remove_dir_all("test-storage/output_preparation").unwrap_or(());
    Ok(())
}

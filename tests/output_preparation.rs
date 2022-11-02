// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::block::address::Address;
use iota_wallet::{
    account::{Assets, Features, OutputOptions},
    account_manager::AccountManager,
    iota_client::{
        block::output::{NativeToken, NftId, TokenId},
        constants::SHIMMER_COIN_TYPE,
    },
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result, U256,
};

#[tokio::test]
async fn output_preparation() -> Result<()> {
    std::fs::remove_dir_all("test-storage/output_preparation").unwrap_or(());
    let client_options = ClientOptions::new()
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled();

    // mnemonic without balance
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(
        "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
    )?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
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
    assert_eq!(output.amount(), 46800);
    // address and sdr unlock condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    let sdr = output.unlock_conditions().unwrap().storage_deposit_return().unwrap();
    assert_eq!(sdr.amount(), 46300);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 500000,
                assets: None,
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 500000);
    // only address condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 1);

    let native_token = NativeToken::new(
        TokenId::from_str("0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000")?,
        U256::from(10u32),
    )?;
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 500000,
                assets: Some(Assets {
                    native_tokens: Some(vec![native_token.clone()]),
                    nft_id: None,
                }),
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 500000);
    // only address condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 1);
    assert_eq!(output.native_tokens().unwrap().first(), Some(&native_token));

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 300000,
                assets: None,
                features: Some(Features {
                    metadata: Some("Hello world".to_string()),
                    tag: None,
                    issuer: None,
                    sender: None,
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 300000);
    // only address condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 1);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 12000,
                assets: None,
                features: Some(Features {
                    metadata: Some("Hello world".to_string()),
                    tag: None,
                    issuer: None,
                    sender: None,
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 54600);
    // address and storage deposit unlock condition, because of the metadata feature block, 12000 is not enough for the
    // required storage deposit
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 1,
                assets: None,
                features: Some(Features {
                    metadata: Some("Hello world".to_string()),
                    tag: None,
                    issuer: None,
                    sender: None,
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.amount(), 48200);
    let sdr = output.unlock_conditions().unwrap().storage_deposit_return().unwrap();
    assert_eq!(sdr.amount(), 48200);

    // address and storage deposit unlock condition, because of the metadata feature block, 213000 is not enough for the
    // required storage deposit
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    // only works if the nft for this NftId is available in the account
    if let Ok(output) = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 500000,
                assets: Some(Assets {
                    native_tokens: None,
                    nft_id: Some(NftId::from_str(
                        "0xa068e00a79922eaef241592a7440f131ea7f8ad9e22e580ef139415f273eff30",
                    )?),
                }),
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await
    {
        assert_eq!(output.kind(), iota_wallet::iota_client::block::output::NftOutput::KIND);
        assert_eq!(output.amount(), 500000);
        // only address condition
        assert_eq!(output.unlock_conditions().unwrap().len(), 1);
    }

    let issuer_and_sender_address = String::from("rms1qq724zgvdujt3jdcd3xzsuqq7wl9pwq3dvsa5zvx49rj9tme8cat6qptyfm");
    if let Ok(output) = account
        .prepare_output(
            OutputOptions {
                recipient_address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
                amount: 500000,
                assets: Some(Assets {
                    native_tokens: None,
                    nft_id: Some(NftId::from_str(
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    )?),
                }),
                features: Some(Features {
                    metadata: None,
                    tag: None,
                    issuer: Some(issuer_and_sender_address.clone()),
                    sender: Some(issuer_and_sender_address.clone()),
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await
    {
        assert_eq!(output.kind(), iota_wallet::iota_client::block::output::NftOutput::KIND);
        assert_eq!(output.amount(), 500000);
        let features = output.features().unwrap();
        // sender and issuer feature
        assert_eq!(features.len(), 2);
        let issuer_and_sender_address = Address::try_from_bech32(&issuer_and_sender_address)?.1;
        let issuer_feature = features.issuer().unwrap();
        assert_eq!(issuer_feature.address(), &issuer_and_sender_address);
        let sender_feature = features.sender().unwrap();
        assert_eq!(sender_feature.address(), &issuer_and_sender_address)
    }

    std::fs::remove_dir_all("test-storage/output_preparation").unwrap_or(());
    Ok(())
}

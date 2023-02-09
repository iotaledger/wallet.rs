// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[cfg(feature = "storage")]
use std::str::FromStr;

#[cfg(feature = "storage")]
use iota_client::block::address::Address;
#[cfg(feature = "storage")]
use iota_wallet::{
    account::{Assets, Features, OutputOptions, Unlocks},
    iota_client::block::output::{NativeToken, NftId, TokenId},
    Result, U256,
};

#[cfg(feature = "storage")]
#[tokio::test]
async fn output_preparation() -> Result<()> {
    let storage_path = "test-storage/output_preparation";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let account = manager.create_account().finish().await?;

    let recipient_address = String::from("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu");
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
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
                recipient_address: recipient_address.clone(),
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
                recipient_address: recipient_address.clone(),
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
                recipient_address: recipient_address.clone(),
                amount: 300000,
                assets: None,
                features: Some(Features {
                    metadata: Some(prefix_hex::encode(b"Hello world")),
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

    // only send 1 with metadata feature
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 1,
                assets: None,
                features: Some(Features {
                    metadata: Some(prefix_hex::encode(b"Hello world")),
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
    let unlock_conditions = output.unlock_conditions().unwrap();
    // address + sdr
    assert_eq!(unlock_conditions.len(), 2);
    let storage_deposit_return = unlock_conditions.storage_deposit_return().unwrap();
    // output amount -1
    assert_eq!(storage_deposit_return.amount(), 48199);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 12000,
                assets: None,
                features: Some(Features {
                    metadata: Some(prefix_hex::encode(b"Hello world")),
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
    // address and storage deposit unlock condition, because of the metadata feature block, 12000 is not enough for the
    // required storage deposit
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 1,
                assets: None,
                features: Some(Features {
                    metadata: Some(prefix_hex::encode(b"Hello world")),
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
    assert_eq!(sdr.amount(), 48199);

    // address and storage deposit unlock condition, because of the metadata feature block, 213000 is not enough for the
    // required storage deposit
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    // metadata feature
    assert_eq!(output.features().unwrap().len(), 1);

    // Error if this NftId is not in the account
    let error = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
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
        .unwrap_err();
    match error {
        iota_wallet::Error::NftNotFoundInUnspentOutputs => {}
        _ => panic!("should return NftNotFoundInUnspentOutputs error"),
    }

    if let Ok(output) = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 500000,
                assets: Some(Assets {
                    native_tokens: None,
                    nft_id: Some(NftId::from_str(
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
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
    let expected_address = Address::try_from_bech32(&issuer_and_sender_address)?.1;

    // sender address present when building basic output
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 500000,
                assets: Some(Assets {
                    native_tokens: Some(vec![native_token.clone()]),
                    nft_id: None,
                }),
                features: Some(Features {
                    metadata: None,
                    tag: None,
                    issuer: None,
                    sender: Some(issuer_and_sender_address.clone()),
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;

    assert_eq!(
        output.kind(),
        iota_wallet::iota_client::block::output::BasicOutput::KIND
    );
    assert_eq!(output.amount(), 500000);
    assert_eq!(output.unlock_conditions().unwrap().len(), 1);
    let features = output.features().unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features.sender().unwrap().address(), &expected_address);

    // error when adding issuer when building basic output
    let error = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 500000,
                assets: None,
                features: Some(Features {
                    metadata: None,
                    tag: None,
                    issuer: Some(issuer_and_sender_address.clone()),
                    sender: None,
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await
        .unwrap_err();
    match error {
        iota_wallet::Error::MissingParameter(_) => {}
        _ => panic!("should return MissingParameter error"),
    }

    // issuer and sender address present when building nft output
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
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
        .await?;
    assert_eq!(output.kind(), iota_wallet::iota_client::block::output::NftOutput::KIND);
    assert_eq!(output.amount(), 500000);
    let features = output.features().unwrap();
    // sender feature
    assert_eq!(features.len(), 1);
    let immutable_features = output.immutable_features().unwrap();
    // issuer feature
    assert_eq!(immutable_features.len(), 1);
    let issuer_and_sender_address = Address::try_from_bech32(&issuer_and_sender_address)?.1;
    let issuer_feature = immutable_features.issuer().unwrap();
    assert_eq!(issuer_feature.address(), &issuer_and_sender_address);
    let sender_feature = features.sender().unwrap();
    assert_eq!(sender_feature.address(), &issuer_and_sender_address);

    // nft with expiration
    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 500,
                assets: Some(Assets {
                    native_tokens: None,
                    nft_id: Some(NftId::from_str(
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    )?),
                }),
                features: Some(Features {
                    metadata: None,
                    tag: None,
                    issuer: None,
                    sender: None,
                }),
                unlocks: Some(Unlocks {
                    expiration_unix_time: Some(1),
                    timelock_unix_time: None,
                }),
                // unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    assert_eq!(output.kind(), iota_wallet::iota_client::block::output::NftOutput::KIND);
    assert_eq!(output.amount(), 53900);
    // address, sdr, expiration
    assert_eq!(output.unlock_conditions().unwrap().len(), 3);

    common::tear_down(storage_path)
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use std::str::FromStr;

use iota_client::block::address::Address;
use iota_wallet::{
    account::{Assets, Features, OutputOptions, Unlocks},
    iota_client::block::output::{NativeToken, NftId, TokenId},
    NftOptions, Result, U256,
};

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
    assert_eq!(output.amount(), 54600);
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

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 8001,
                assets: None,
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    let rent_structure = account.client().get_rent_structure().await?;
    let token_supply = account.client().get_token_supply().await?;
    // Check if the output has enough amount to cover the storage deposit
    output.verify_storage_deposit(rent_structure, token_supply)?;
    assert_eq!(output.amount(), 50601);
    // address and sdr unlock condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    let sdr = output.unlock_conditions().unwrap().storage_deposit_return().unwrap();
    assert_eq!(sdr.amount(), 42600);

    let output = account
        .prepare_output(
            OutputOptions {
                recipient_address: recipient_address.clone(),
                amount: 42599,
                assets: None,
                features: None,
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?;
    let rent_structure = account.client().get_rent_structure().await?;
    let token_supply = account.client().get_token_supply().await?;
    // Check if the output has enough amount to cover the storage deposit
    output.verify_storage_deposit(rent_structure, token_supply)?;
    assert_eq!(output.amount(), 85199);
    // address and sdr unlock condition
    assert_eq!(output.unlock_conditions().unwrap().len(), 2);
    let sdr = output.unlock_conditions().unwrap().storage_deposit_return().unwrap();
    assert_eq!(sdr.amount(), 42600);

    common::tear_down(storage_path)
}

#[ignore]
#[tokio::test]
async fn prepare_nft_output_features_update() -> Result<()> {
    let storage_path = "test-storage/output_preparation";
    common::setup(storage_path)?;

    let manager = common::make_manager(storage_path, None, None).await?;
    let accounts = &common::create_accounts_with_funds(&manager, 1).await?;
    let address = accounts[0].addresses().await?[0].address().to_bech32();

    let nft_options = vec![NftOptions {
        address: Some(address.clone()),
        sender: Some(address.clone()),
        metadata: Some(b"some nft metadata".to_vec()),
        tag: Some(b"some nft tag".to_vec()),
        issuer: Some(address.clone()),
        immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    }];

    let transaction = accounts[0].mint_nfts(nft_options, None).await.unwrap();
    accounts[0]
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;

    let nft_id = *accounts[0].sync(None).await?.nfts.first().unwrap();

    let nft = accounts[0]
        .prepare_output(
            OutputOptions {
                recipient_address: address,
                amount: 1_000_000,
                assets: Some(Assets {
                    native_tokens: None,
                    nft_id: Some(nft_id),
                }),
                features: Some(Features {
                    metadata: Some("0x2a".to_string()),
                    tag: None,
                    issuer: None,
                    sender: None,
                }),
                unlocks: None,
                storage_deposit: None,
            },
            None,
        )
        .await?
        .as_nft()
        .clone();

    assert_eq!(nft.amount(), 1_000_000);
    assert_eq!(nft.address(), accounts[0].addresses().await?[0].address().as_ref());
    assert!(nft.features().sender().is_none());
    assert!(nft.features().tag().is_none());
    assert_eq!(nft.features().metadata().unwrap().data(), &[42]);
    assert_eq!(
        nft.immutable_features().metadata().unwrap().data(),
        b"some immutable nft metadata"
    );
    assert_eq!(
        nft.immutable_features().issuer().unwrap().address(),
        accounts[0].addresses().await?[0].address().as_ref()
    );

    common::tear_down(storage_path)
}

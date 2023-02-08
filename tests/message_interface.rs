// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[cfg(feature = "message_interface")]
use std::sync::{atomic::Ordering, Arc};

#[cfg(feature = "message_interface")]
use iota_client::{
    block::{
        address::Address,
        output::{
            dto::OutputDto,
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            BasicOutputBuilder,
        },
    },
    constants::SHIMMER_COIN_TYPE,
    ClientBuilder,
};
#[cfg(feature = "events")]
use iota_wallet::events::types::WalletEvent;
#[cfg(feature = "message_interface")]
use iota_wallet::{
    message_interface::{create_message_handler, send_message, AccountMethod, ManagerOptions, Message, Response},
    Result,
};

#[cfg(feature = "message_interface")]
const TOKEN_SUPPLY: u64 = 1_813_620_509_061_365;

#[cfg(feature = "message_interface")]
#[tokio::test]
async fn message_interface_validate_mnemonic() -> Result<()> {
    let storage_path = "test-storage/message_interface_validate_mnemonic";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
    let client_options = r#"{"nodes":["http://localhost:14265"]}"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();
    let response = send_message(&wallet_handle, Message::GenerateMnemonic)
        .await
        .expect("No send message response");

    match response {
        Response::GeneratedMnemonic(mnemonic) => {
            let response = send_message(
                &wallet_handle,
                Message::VerifyMnemonic {
                    mnemonic: mnemonic.to_string(),
                },
            )
            .await
            .expect("No send message response");

            let Response::Ok(_) = response else {
                panic!("Unexpected response type: {response:?}");
            };
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    common::tear_down(storage_path)
}

#[cfg(feature = "message_interface")]
#[tokio::test]
async fn message_interface_create_account() -> Result<()> {
    let storage_path = "test-storage/message_interface_create_account";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"about solution utility exist rail budget vacuum major survey clerk pave ankle wealth gym gossip still medal expect strong rely amazing inspire lazy lunar"}"#;
    let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265",
                  "auth":null,
                  "disabled":false
               },
               {
                  "url":"https://api.testnet.shimmer.network",
                  "auth":null,
                  "disabled":false
               }
            ],
            "localPow":true,
            "apiTimeout":{
               "secs":20,
               "nanos":0
            }
         }"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    // create an account
    let response = send_message(
        &wallet_handle,
        Message::CreateAccount {
            alias: None,
            bech32_hrp: None,
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::Account(account) => {
            assert_eq!(account.index, 0);
            let id = account.index;
            println!("Created account index: {id}")
        }
        _ => panic!("unexpected response {response:?}"),
    }

    common::tear_down(storage_path)
}

#[cfg(all(feature = "message_interface", feature = "events"))]
#[tokio::test]
async fn message_interface_events() -> Result<()> {
    let storage_path = "test-storage/message_interface_events";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"member captain exotic police quit giraffe oval album proof skin fame cannon soccer cinnamon gaze mango fetch identify vocal cause drink stem produce twice"}"#;
    let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    wallet_handle
        .listen(vec![], |event| {
            if let WalletEvent::TransactionProgress(event) = &event.event {
                println!("Received event....: {event:?}");
            }
        })
        .await;

    // create an account
    let response = send_message(
        &wallet_handle,
        Message::CreateAccount {
            alias: Some("alias".to_string()),
            bech32_hrp: None,
        },
    )
    .await
    .unwrap();

    match response {
        Response::Account(account) => {
            assert_eq!(account.index, 0);

            // get funds from faucet
            let transaction = Message::CallAccountMethod {
                account_id: "alias".into(),
                method: AccountMethod::RequestFundsFromFaucet {
                    url: common::FAUCET_URL.to_string(),
                    address: account.public_addresses[0].address().to_bech32(),
                },
            };

            let _response = send_message(&wallet_handle, transaction).await.unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        }
        _ => panic!("unexpected response {response:?}"),
    }

    // sync the account
    let sync_method = Message::CallAccountMethod {
        account_id: "alias".into(),
        method: AccountMethod::SyncAccount { options: None },
    };

    let _response = send_message(&wallet_handle, sync_method).await.unwrap();

    // send transaction
    let outputs = vec![OutputDto::from(
        &BasicOutputBuilder::new_with_amount(1_000_000)
            .unwrap()
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                Address::try_from_bech32("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu")
                    .unwrap()
                    .1,
            )))
            .finish_output(TOKEN_SUPPLY)
            .unwrap(),
    )];

    let transaction = Message::CallAccountMethod {
        account_id: "alias".into(),
        method: AccountMethod::SendOutputs { outputs, options: None },
    };

    let response = send_message(&wallet_handle, transaction).await.unwrap();

    let Response::SentTransaction(_)= response else {
        panic!("unexpected response {response:?}");
    };

    common::tear_down(storage_path)
}

#[cfg(all(feature = "message_interface", feature = "events"))]
#[tokio::test]
async fn message_interface_emit_event() -> Result<()> {
    let storage_path = "test-storage/message_interface_emit_event";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"father defy final warm illness local fetch property staff boss diamond icon burger people lemon scene silent slender never vacant lab lazy tube tomato"}"#;
    let client_options = r#"{"nodes":["http://localhost:14265"]}"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    let event_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let event_counter_clone = Arc::clone(&event_counter);
    wallet_handle
        .listen(vec![], move |_name| {
            event_counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await;

    for count in 1..11 {
        let response = send_message(
            &wallet_handle,
            Message::EmitTestEvent {
                event: WalletEvent::ConsolidationRequired,
            },
        )
        .await
        .expect("No send message response");
        match response {
            Response::Ok(()) => {
                assert_eq!(count, event_counter.load(Ordering::SeqCst))
            }
            response_type => panic!("Unexpected response type: {response_type:?}"),
        }
        dbg!(&count);
    }

    send_message(&wallet_handle, Message::ClearListeners { event_types: vec![] }).await;
    send_message(
        &wallet_handle,
        Message::EmitTestEvent {
            event: WalletEvent::ConsolidationRequired,
        },
    )
    .await
    .expect("No send message response");

    // Event should not have fired, so we are still on 10 calls
    assert_eq!(10, event_counter.load(Ordering::SeqCst));

    common::tear_down(storage_path)
}

#[cfg(all(feature = "message_interface", feature = "stronghold"))]
#[tokio::test]
async fn message_interface_stronghold() -> Result<()> {
    let storage_path = "test-storage/message_interface_stronghold";
    common::setup(storage_path)?;
    let snapshot_path = "test-storage/message_interface_stronghold/message_interface.stronghold";
    let secret_manager = format!("{{\"Stronghold\": {{\"snapshotPath\": \"{snapshot_path}\"}}}}");

    let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(&secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    // Set password and store mnemonic
    let _ = send_message(
        &wallet_handle,
        Message::SetStrongholdPassword {
            password: "some_hopefully_secure_password".to_string(),
        },
    )
    .await;
    let mnemonic = "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast".to_string();
    let _ = send_message(&wallet_handle, Message::StoreMnemonic { mnemonic }).await;

    // create an account, if password or storing mnemonic failed, it would fail here, because it couldn't generate
    // an address
    let response = send_message(
        &wallet_handle,
        Message::CreateAccount {
            alias: None,
            bech32_hrp: None,
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::Account(account) => {
            let id = account.index;
            println!("Created account index: {id}")
        }
        _ => panic!("unexpected response {response:?}"),
    }

    common::tear_down(storage_path)
}

#[cfg(feature = "message_interface")]
#[tokio::test]
async fn address_conversion_methods() -> Result<()> {
    let storage_path = "test-storage/address_conversion_methods";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
    let client_options = r#"{"nodes":["http://localhost:14265"]}"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    let bech32_address = "rms1qqk4svqpc89lxx89w7vksv9jgjjm2vwnrhad2j3cds9ev4cu434wjapdsxs";
    let hex_address = "0x2d583001c1cbf318e577996830b244a5b531d31dfad54a386c0b96571cac6ae9";

    let response = send_message(
        &wallet_handle,
        Message::Bech32ToHex {
            bech32_address: bech32_address.into(),
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::HexAddress(hex) => {
            assert_eq!(hex, hex_address);
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    let response = send_message(
        &wallet_handle,
        Message::HexToBech32 {
            hex: hex_address.into(),
            bech32_hrp: None,
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::Bech32Address(bech32) => {
            assert_eq!(bech32, bech32_address);
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    common::tear_down(storage_path)
}

#[cfg(feature = "message_interface")]
#[tokio::test]
async fn message_interface_address_generation() -> Result<()> {
    let storage_path = "test-storage/message_interface_address_generation";
    common::setup(storage_path)?;

    let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
    let client_options = r#"{"nodes":["http://localhost:14265"]}"#;

    let options = ManagerOptions {
        #[cfg(feature = "storage")]
        storage_path: Some(storage_path.to_string()),
        client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
        coin_type: Some(SHIMMER_COIN_TYPE),
        secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
    };

    let wallet_handle = create_message_handler(Some(options)).await.unwrap();

    let response = send_message(
        &wallet_handle,
        Message::GenerateAddress {
            account_index: 0,
            internal: false,
            address_index: 0,
            options: None,
            bech32_hrp: Some("rms".to_string()),
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::Bech32Address(address) => {
            assert_eq!(
                address,
                "rms1qzev36lk0gzld0k28fd2fauz26qqzh4hd4cwymlqlv96x7phjxcw6v3ea5a"
            );
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    let response = send_message(
        &wallet_handle,
        Message::GenerateAddress {
            account_index: 10,
            internal: true,
            address_index: 10,
            options: None,
            bech32_hrp: Some("rms".to_string()),
        },
    )
    .await
    .expect("No send message response");

    match response {
        Response::Bech32Address(address) => {
            assert_eq!(
                address,
                "rms1qr239vcjzxxdyre8jsek8wrdves9hnnk6mguplvs43cwftt4svaszsvy98h"
            );
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    common::tear_down(storage_path)
}

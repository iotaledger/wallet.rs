// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[cfg(feature = "message_interface")]
use std::{
    str::FromStr,
    sync::{atomic::Ordering, Arc},
};

#[cfg(feature = "message_interface")]
use iota_client::{
    block::{
        address::Address,
        output::{
            dto::OutputDto,
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            BasicOutputBuilder,
        },
        BlockDto, BlockId,
    },
    constants::SHIMMER_COIN_TYPE,
    message_interface::{Message as ClientMessage, Response as ClientResponse},
    ClientBuilder,
};
#[cfg(feature = "events")]
use iota_wallet::events::types::WalletEvent;
#[cfg(feature = "message_interface")]
use iota_wallet::{
    message_interface::{create_message_handler, AccountMethod, ManagerOptions, Message, Response},
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
    let response = wallet_handle.send_message(Message::GenerateMnemonic).await;

    match response {
        Response::GeneratedMnemonic(mnemonic) => {
            let response = wallet_handle
                .send_message(Message::VerifyMnemonic {
                    mnemonic: mnemonic.to_string(),
                })
                .await;

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
    let response = wallet_handle
        .send_message(Message::CreateAccount {
            alias: None,
            bech32_hrp: None,
        })
        .await;

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

#[ignore]
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
    let response = wallet_handle
        .send_message(Message::CreateAccount {
            alias: Some("alias".to_string()),
            bech32_hrp: None,
        })
        .await;

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

            let _response = wallet_handle.send_message(transaction).await;

            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        }
        _ => panic!("unexpected response {response:?}"),
    }

    // sync the account
    let sync_method = Message::CallAccountMethod {
        account_id: "alias".into(),
        method: AccountMethod::SyncAccount { options: None },
    };

    let _response = wallet_handle.send_message(sync_method).await;

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

    let response = wallet_handle.send_message(transaction).await;

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
        let response = wallet_handle
            .send_message(Message::EmitTestEvent {
                event: WalletEvent::ConsolidationRequired,
            })
            .await;
        match response {
            Response::Ok(()) => {
                assert_eq!(count, event_counter.load(Ordering::SeqCst))
            }
            response_type => panic!("Unexpected response type: {response_type:?}"),
        }
        dbg!(&count);
    }

    wallet_handle
        .send_message(Message::ClearListeners { event_types: vec![] })
        .await;
    wallet_handle
        .send_message(Message::EmitTestEvent {
            event: WalletEvent::ConsolidationRequired,
        })
        .await;

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
    let _ = wallet_handle
        .send_message(Message::SetStrongholdPassword {
            password: "some_hopefully_secure_password".to_string(),
        })
        .await;
    let mnemonic = "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast".to_string();
    let _ = wallet_handle.send_message(Message::StoreMnemonic { mnemonic }).await;

    // create an account, if password or storing mnemonic failed, it would fail here, because it couldn't generate
    // an address
    let response = wallet_handle
        .send_message(Message::CreateAccount {
            alias: None,
            bech32_hrp: None,
        })
        .await;

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

    let response = wallet_handle
        .send_message(Message::Bech32ToHex {
            bech32_address: bech32_address.into(),
        })
        .await;

    match response {
        Response::HexAddress(hex) => {
            assert_eq!(hex, hex_address);
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    let response = wallet_handle
        .send_message(Message::HexToBech32 {
            hex: hex_address.into(),
            bech32_hrp: None,
        })
        .await;

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

    let response = wallet_handle
        .send_message(Message::GenerateAddress {
            account_index: 0,
            internal: false,
            address_index: 0,
            options: None,
            bech32_hrp: Some("rms".to_string()),
        })
        .await;

    match response {
        Response::Bech32Address(address) => {
            assert_eq!(
                address,
                "rms1qzev36lk0gzld0k28fd2fauz26qqzh4hd4cwymlqlv96x7phjxcw6v3ea5a"
            );
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    let response = wallet_handle
        .send_message(Message::GenerateAddress {
            account_index: 10,
            internal: true,
            address_index: 10,
            options: None,
            bech32_hrp: Some("rms".to_string()),
        })
        .await;

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

#[cfg(feature = "message_interface")]
#[tokio::test]
async fn client_message_interface() -> Result<()> {
    let storage_path = "test-storage/client_message_interface";
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

    let block = r#"
    {
        "protocolVersion":2,
        "parents":
            [
                "0x2881c4781c4126f2413a704ebdf8cd375b46007f8df0e32ee9158684ac7e307b",
                "0xe1956a33d608cb2bcfd6adeb67fe56ed0f33fc5ffd157e28a71047ecc52b0314",
                "0xecc442108b1f30b6208ea57d24d892a6bdbdd9eb068dd34640a4d38b3c757132",
                "0xfad7cc342cfa1135f9c12e99f98ec1658ec178524d19bde7b4797d81cecf9ea6"
            ],
        "payload":
            {
                "type":5,
                "tag":"0x484f524e4554205370616d6d6572",
                "data":"0x494f5441202d2041206e6577206461776e0a436f756e743a203030323330330a54696d657374616d703a20323032322d30342d32375431383a35343a30395a0a54697073656c656374696f6e3a203832c2b573"
            },
        "nonce":"22897"
    }"#;

    let block_dto: BlockDto = serde_json::from_str(block).unwrap();
    let message = Message::Client {
        message: ClientMessage::BlockId { block: block_dto },
    };

    let response = wallet_handle.send_message(message).await;

    match response {
        Response::Client(ClientResponse::BlockId(block_id)) => {
            assert_eq!(
                block_id,
                BlockId::from_str("0xbcd2b9feed097a7aa8b894cae5eaeb1d8f516a14af25aa6f7d8aa7e2604c406c").unwrap()
            );
        }
        response_type => panic!("Unexpected response type: {response_type:?}"),
    }

    common::tear_down(storage_path)
}

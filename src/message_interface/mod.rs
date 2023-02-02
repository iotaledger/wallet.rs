// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account_method;
pub mod dtos;
mod message;
mod message_handler;
mod response;

use fern_logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_client::secret::{SecretManager, SecretManagerDto};
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::mpsc::unbounded_channel;

pub use self::{
    account_method::AccountMethod,
    dtos::{AddressWithAmountDto, AddressWithUnspentOutputsDto},
    message::Message,
    message_handler::WalletMessageHandler,
    response::Response,
};
use crate::{account_manager::AccountManager, ClientOptions};

#[derive(Serialize, Deserialize, Debug)]
pub struct ManagerOptions {
    #[serde(rename = "storagePath")]
    storage_path: Option<String>,
    #[serde(rename = "clientOptions")]
    client_options: Option<ClientOptions>,
    #[serde(rename = "coinType")]
    pub coin_type: Option<u32>,
    #[serde(rename = "secretManager", serialize_with = "secret_manager_serialize")]
    secret_manager: Option<SecretManagerDto>,
}

// Serialize secret manager with secrets removed
fn secret_manager_serialize<S>(secret_manager: &Option<SecretManagerDto>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(secret_manager) = secret_manager {
        match secret_manager {
            SecretManagerDto::HexSeed(_) => s.serialize_str("hexSeed(<omitted>)"),
            #[cfg(feature = "ledger_nano")]
            SecretManagerDto::LedgerNano(is_simulator) => s.serialize_str(&format!("ledgerNano({is_simulator})")),
            SecretManagerDto::Mnemonic(_) => s.serialize_str("mnemonic(<omitted>)"),
            SecretManagerDto::Placeholder => s.serialize_str("placeholder"),
            #[cfg(feature = "stronghold")]
            SecretManagerDto::Stronghold(stronghold) => {
                let mut stronghold_dto = stronghold.clone();
                // Remove password
                stronghold_dto.password = None;
                s.serialize_str(&format!("{stronghold_dto:?}"))
            }
        }
    } else {
        s.serialize_str("null")
    }
}

pub fn init_logger(config: String) -> Result<(), fern_logger::Error> {
    let output_config: LoggerOutputConfigBuilder = serde_json::from_str(&config).expect("invalid logger config");
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config)
}

pub async fn create_message_handler(options: Option<ManagerOptions>) -> crate::Result<WalletMessageHandler> {
    log::debug!(
        "create_message_handler with options: {}",
        serde_json::to_string(&options)?,
    );
    let manager = if let Some(options) = options {
        let mut builder = AccountManager::builder();

        #[cfg(feature = "storage")]
        if let Some(storage_path) = options.storage_path {
            builder = builder.with_storage_path(&storage_path);
        }

        if let Some(secret_manager) = options.secret_manager {
            builder = builder.with_secret_manager(SecretManager::try_from(&secret_manager)?);
        }

        if let Some(client_options) = options.client_options {
            builder = builder.with_client_options(client_options);
        }

        if let Some(coin_type) = options.coin_type {
            builder = builder.with_coin_type(coin_type);
        }

        builder.finish().await?
    } else {
        AccountManager::builder().finish().await?
    };

    Ok(WalletMessageHandler::with_manager(manager))
}

pub async fn send_message(handle: &WalletMessageHandler, message: Message) -> Option<Response> {
    let (message_tx, mut message_rx) = unbounded_channel();
    handle.handle(message, message_tx).await;
    message_rx.recv().await
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{atomic::Ordering, Arc},
    };

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
    use crate::events::types::WalletEvent;
    use crate::message_interface::{self, AccountMethod, ManagerOptions, Message, Response};

    const TOKEN_SUPPLY: u64 = 1_813_620_509_061_365;

    #[tokio::test]
    #[cfg(feature = "events")]
    async fn message_interface_emit_event() {
        let storage_path = "test-storage/message_interface_emit_event";
        std::fs::remove_dir_all(storage_path).unwrap_or(());

        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
        let client_options = r#"{"nodes":["http://localhost:14265/"]}"#;

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some(storage_path.to_string()),
            client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        let event_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let event_counter_clone = Arc::clone(&event_counter);
        wallet_handle
            .listen(vec![], move |_name| {
                event_counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .await;

        for count in 1..11 {
            let response = message_interface::send_message(
                &wallet_handle,
                Message::EmitTestEvent(WalletEvent::ConsolidationRequired),
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

        message_interface::send_message(&wallet_handle, Message::ClearListeners(vec![])).await;
        message_interface::send_message(
            &wallet_handle,
            Message::EmitTestEvent(WalletEvent::ConsolidationRequired),
        )
        .await
        .expect("No send message response");

        // Event should not have fired, so we are still on 10 calls
        assert_eq!(10, event_counter.load(Ordering::SeqCst));

        std::fs::remove_dir_all(storage_path).unwrap_or(());
    }

    #[tokio::test]
    async fn message_interface_validate_mnemonic() {
        let storage_path = "test-storage/message_interface_validate_mnemonic";
        std::fs::remove_dir_all(storage_path).unwrap_or(());

        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
        let client_options = r#"{"nodes":["http://localhost:14265/"]}"#;

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some(storage_path.to_string()),
            client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();
        let response = message_interface::send_message(&wallet_handle, Message::GenerateMnemonic)
            .await
            .expect("No send message response");

        match response {
            Response::GeneratedMnemonic(mnemonic) => {
                let response =
                    message_interface::send_message(&wallet_handle, Message::VerifyMnemonic(mnemonic.to_string()))
                        .await
                        .expect("No send message response");

                match response {
                    Response::Ok(()) => {}
                    response_type => panic!("Unexpected response type: {response_type:?}"),
                }
            }
            response_type => panic!("Unexpected response type: {response_type:?}"),
        }

        std::fs::remove_dir_all(storage_path).unwrap_or(());
    }

    #[tokio::test]
    async fn message_interface_create_account() {
        let storage_path = "test-storage/message_interface_create_account";
        std::fs::remove_dir_all(storage_path).unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
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

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // create an account
        let response = message_interface::send_message(
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

        std::fs::remove_dir_all(storage_path).unwrap_or(());
    }

    #[cfg(feature = "events")]
    #[tokio::test]
    async fn message_interface_events() {
        std::fs::remove_dir_all("test-storage/message_interface_events").unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#;

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/message_interface_events".to_string()),
            client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        wallet_handle
            .listen(vec![], |event| {
                if let WalletEvent::TransactionProgress(event) = &event.event {
                    println!("Received event....: {event:?}");
                }
            })
            .await;

        // create an account
        let _ = message_interface::send_message(
            &wallet_handle,
            Message::CreateAccount {
                alias: Some("alias".to_string()),
                bech32_hrp: None,
            },
        )
        .await;

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

        let _response = message_interface::send_message(&wallet_handle, transaction).await;

        std::fs::remove_dir_all("test-storage/message_interface_events").unwrap_or(());
    }

    #[cfg(feature = "stronghold")]
    #[tokio::test]
    async fn message_interface_stronghold() {
        std::fs::remove_dir_all("test-storage/message_interface_stronghold").unwrap_or(());
        let snapshot_path = "message_interface.stronghold";
        let secret_manager = format!("{{\"Stronghold\": {{\"snapshotPath\": \"{snapshot_path}\"}}}}");

        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#;

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/message_interface_stronghold".to_string()),
            client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(serde_json::from_str(&secret_manager).unwrap()),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // Set password and store mnemonic
        let _ = message_interface::send_message(
            &wallet_handle,
            Message::SetStrongholdPassword {
                password: "some_hopefully_secure_password".to_string(),
            },
        )
        .await;
        let mnemonic = "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast".to_string();
        let _ = message_interface::send_message(&wallet_handle, Message::StoreMnemonic { mnemonic }).await;

        // create an account, if password or storing mnemonic failed, it would fail here, because it couldn't generate
        // an address
        let response = message_interface::send_message(
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

        std::fs::remove_dir_all("test-storage/message_interface_stronghold").unwrap_or(());
        fs::remove_file(snapshot_path).unwrap();
    }

    #[tokio::test]
    async fn address_conversion_methods() {
        std::fs::remove_dir_all("test-storage/address_conversion_methods").unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#;
        let client_options = r#"{"nodes":["http://localhost:14265/"]}"#;

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/address_conversion_methods".to_string()),
            client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        let bech32_address = "rms1qqk4svqpc89lxx89w7vksv9jgjjm2vwnrhad2j3cds9ev4cu434wjapdsxs";
        let hex_address = "0x2d583001c1cbf318e577996830b244a5b531d31dfad54a386c0b96571cac6ae9";

        let response = message_interface::send_message(
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

        let response = message_interface::send_message(
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

        std::fs::remove_dir_all("test-storage/address_conversion_methods").unwrap_or(());
    }
}

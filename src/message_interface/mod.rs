// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account_method;
pub mod dtos;
mod message;
mod message_handler;
mod response;

use std::str::FromStr;

use iota_client::secret::SecretManager;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::mpsc::unbounded_channel;

pub use self::{
    account_method::AccountMethod,
    dtos::{AccountBalanceDto, AddressWithAmountDto, AddressWithUnspentOutputsDto},
    message::{AccountToCreate, Message},
    message_handler::WalletMessageHandler,
    response::Response,
};
#[cfg(feature = "events")]
use crate::events::types::{Event, WalletEventType};
use crate::{account_manager::AccountManager, ClientOptions};

#[derive(Serialize, Deserialize, Debug)]
pub struct ManagerOptions {
    #[serde(rename = "storagePath")]
    storage_path: Option<String>,
    #[serde(rename = "clientOptions")]
    client_options: Option<String>,
    #[serde(rename = "coinType")]
    pub coin_type: Option<u32>,
    #[serde(rename = "secretManager", serialize_with = "secret_manager_serialize")]
    secret_manager: Option<String>,
}

// Don't serialize the secret_manager, because we don't want to log the mnemonic or password
fn secret_manager_serialize<S>(x: &Option<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("is_some: {}", x.is_some()))
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
            builder = builder.with_secret_manager(SecretManager::from_str(&secret_manager)?);
        }

        if let Some(client_options) = options.client_options {
            builder = builder.with_client_options(ClientOptions::new().from_json(&client_options)?);
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

pub async fn send_message(handle: &WalletMessageHandler, message: Message) -> Response {
    let (message_tx, mut message_rx) = unbounded_channel();
    handle.handle(message, message_tx).await;
    message_rx.recv().await.unwrap()
}

#[cfg(feature = "events")]
/// Listen to wallet events, empty vec will listen to all events
pub async fn listen<F>(handle: &WalletMessageHandler, events: Vec<WalletEventType>, handler: F)
where
    F: Fn(&Event) + 'static + Clone + Send + Sync,
{
    handle.listen(events, handler).await;
}

#[cfg(feature = "events")]
/// Remove wallet event listeners, empty vec will remove all listeners
pub async fn clear_listeners(handle: &WalletMessageHandler, events: Vec<WalletEventType>) {
    handle.clear_listeners(events).await;
}

#[cfg(test)]
mod tests {
    use iota_client::{
        bee_block::{
            address::Address,
            output::{
                dto::OutputDto,
                unlock_condition::{AddressUnlockCondition, UnlockCondition},
                BasicOutputBuilder,
            },
        },
        constants::SHIMMER_COIN_TYPE,
    };

    #[cfg(feature = "events")]
    use crate::events::types::WalletEvent;
    use crate::message_interface::{self, AccountMethod, AccountToCreate, ManagerOptions, Message, Response};

    #[tokio::test]
    async fn message_interface_create_account() {
        std::fs::remove_dir_all("test-storage/message_interface_create_account").unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#.to_string();
        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               },
               {
                  "url":"https://chrysalis-nodes.iota.cafe/",
                  "auth":null,
                  "disabled":false
               }
            ],
            "localPow":true,
            "apiTimeout":{
               "secs":20,
               "nanos":0
            }
         }"#
        .to_string();

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/message_interface_create_account".to_string()),
            client_options: Some(client_options),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // create an account
        let account = AccountToCreate { alias: None };
        let response = message_interface::send_message(&wallet_handle, Message::CreateAccount(Box::new(account))).await;
        match response {
            Response::Account(account) => {
                let id = account.index;
                println!("Created account index: {id}")
            }
            _ => panic!("unexpected response {:?}", response),
        }
        std::fs::remove_dir_all("test-storage/message_interface_create_account").unwrap_or(());
    }

    #[cfg(feature = "events")]
    #[tokio::test]
    async fn message_interface_events() {
        std::fs::remove_dir_all("test-storage/message_interface_events").unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#.to_string();
        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#
        .to_string();

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/message_interface_events".to_string()),
            client_options: Some(client_options),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        wallet_handle
            .listen(vec![], |event| {
                if let WalletEvent::TransactionProgress(event) = &event.event {
                    println!("Received event....: {:?}", event);
                }
            })
            .await;

        // create an account
        let account = AccountToCreate {
            alias: Some("alias".to_string()),
        };
        let _ = message_interface::send_message(&wallet_handle, Message::CreateAccount(Box::new(account))).await;

        // send transaction
        let outputs = vec![OutputDto::from(
            &BasicOutputBuilder::new_with_amount(1_000_000)
                .unwrap()
                .add_unlock_condition(
                    UnlockCondition::Address(
                        AddressUnlockCondition::new(
                            Address::try_from_bech32("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu")
                                .unwrap()
                                .1,
                        ),
                    ),
                )
                .finish_output()
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
        let secret_manager = r#"{"Stronghold": {"snapshotPath": "test.stronghold"}}"#.to_string();
        let client_options = r#"{
            "nodes":[
               {
                  "url":"http://localhost:14265/",
                  "auth":null,
                  "disabled":false
               }
            ]
         }"#
        .to_string();

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/message_interface_stronghold".to_string()),
            client_options: Some(client_options),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // Set password and store mnemonic
        let _ = message_interface::send_message(
            &wallet_handle,
            Message::SetStrongholdPassword("some_hopefully_secure_password".to_string()),
        )
        .await;
        let mnemonic = "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast".to_string();
        let _ = message_interface::send_message(&wallet_handle, Message::StoreMnemonic(mnemonic)).await;

        // create an account, if password or storing mnemonic failed, it would fail here, because it couldn't generate
        // an address
        let account = AccountToCreate { ..Default::default() };
        let response = message_interface::send_message(&wallet_handle, Message::CreateAccount(Box::new(account))).await;

        match response {
            Response::Account(account) => {
                let id = account.index;
                println!("Created account index: {id}")
            }
            _ => panic!("unexpected response {:?}", response),
        }
        std::fs::remove_dir_all("test-storage/message_interface_stronghold").unwrap_or(());
    }

    #[tokio::test]
    async fn address_conversion_methods() {
        std::fs::remove_dir_all("test-storage/address_conversion_methods").unwrap_or(());
        let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#.to_string();
        let client_options = r#"{"nodes":["http://localhost:14265/"]}"#.to_string();

        let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_path: Some("test-storage/address_conversion_methods".to_string()),
            client_options: Some(client_options),
            coin_type: Some(SHIMMER_COIN_TYPE),
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        let bech32_address = "rms1qqk4svqpc89lxx89w7vksv9jgjjm2vwnrhad2j3cds9ev4cu434wjapdsxs";
        let hex_address = "0x2d583001c1cbf318e577996830b244a5b531d31dfad54a386c0b96571cac6ae9";

        let response =
            message_interface::send_message(&wallet_handle, Message::Bech32ToHex(bech32_address.into())).await;

        match response {
            Response::HexAddress(hex) => {
                assert_eq!(hex, hex_address);
            }
            response_type => panic!("Unexpected response type: {:?}", response_type),
        }

        let response = message_interface::send_message(
            &wallet_handle,
            Message::HexToBech32 {
                hex: hex_address.into(),
                bech32_hrp: None,
            },
        )
        .await;

        match response {
            Response::Bech32Address(bech32) => {
                assert_eq!(bech32, bech32_address);
            }
            response_type => panic!("Unexpected response type: {:?}", response_type),
        }
        std::fs::remove_dir_all("test-storage/address_conversion_methods").unwrap_or(());
    }
}

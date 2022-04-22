// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account_method;
mod message;
mod message_handler;
mod message_type;
mod response;
mod response_type;

use std::str::FromStr;

use iota_client::secret::SecretManagerType;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::mpsc::unbounded_channel;

pub use self::{
    account_method::AccountMethod,
    message::Message,
    message_handler::WalletMessageHandler,
    message_type::{AccountToCreate, MessageType},
    response::Response,
    response_type::ResponseType,
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
    #[serde(serialize_with = "secret_manager_serialize")]
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
            builder = builder.with_secret_manager(SecretManagerType::from_str(&secret_manager)?);
        }

        if let Some(client_options) = options.client_options {
            builder = builder.with_client_options(ClientOptions::new().from_json(&client_options)?);
        }

        builder.finish().await?
    } else {
        AccountManager::builder().finish().await?
    };

    Ok(WalletMessageHandler::with_manager(manager))
}

pub async fn send_message(handle: &WalletMessageHandler, message_type: MessageType) -> Response {
    let (message_tx, mut message_rx) = unbounded_channel();
    let message = Message::new(message_type, message_tx);
    handle.handle(message).await;
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

#[cfg(test)]
mod tests {
    use iota_client::bee_message::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            BasicOutputBuilder, Output,
        },
    };

    #[cfg(feature = "events")]
    use crate::events::types::WalletEvent;
    use crate::message_interface::{self, AccountMethod, AccountToCreate, ManagerOptions, MessageType, ResponseType};

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
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // create an account
        let account = AccountToCreate { alias: None };
        let response =
            message_interface::send_message(&wallet_handle, MessageType::CreateAccount(Box::new(account))).await;
        match response.response() {
            ResponseType::CreatedAccount(account) => {
                let id = account.index();
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
            secret_manager: Some(secret_manager),
        };

        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        wallet_handle
            .listen(vec![], |event| {
                if let WalletEvent::TransferProgress(event) = &event.event {
                    println!("Received event....: {:?}", event);
                }
            })
            .await;

        // create an account
        let account = AccountToCreate {
            alias: Some("alias".to_string()),
        };
        let _ = message_interface::send_message(&wallet_handle, MessageType::CreateAccount(Box::new(account))).await;

        // send transaction
        let outputs = vec![Output::Basic(
            BasicOutputBuilder::new_with_amount(1_000_000)
                .unwrap()
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    Address::try_from_bech32("atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e")
                        .unwrap()
                        .1,
                )))
                .finish()
                .unwrap(),
        )];

        let transfer = MessageType::CallAccountMethod {
            account_id: "alias".into(),
            method: AccountMethod::SendTransfer { outputs, options: None },
        };

        let _response = message_interface::send_message(&wallet_handle, transfer).await;
        std::fs::remove_dir_all("test-storage/message_interface_events").unwrap_or(());
    }
}

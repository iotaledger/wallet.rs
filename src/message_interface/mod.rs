// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account_method;
mod message;
mod message_handler;
mod message_type;
mod response;
mod response_type;

pub use account_method::AccountMethod;
pub use message::Message;
pub use message_handler::WalletMessageHandler;
pub use message_type::{AccountToCreate, MessageType};
pub use response::Response;
pub use response_type::ResponseType;
use serde::Deserialize;

use crate::{account_manager::AccountManager, client::ClientOptions, Result};
use iota_client::signing::SignerHandle;
use tokio::sync::mpsc::unbounded_channel;

#[derive(Deserialize)]
pub struct ManagerOptions {
    #[cfg(feature = "storage")]
    storage_folder: Option<String>,
    client_options: Option<String>,
    signer: Option<String>,
}

pub async fn create_message_handler(options: Option<ManagerOptions>) -> Result<WalletMessageHandler> {
    let manager = if let Some(options) = options {
        let mut builder = AccountManager::builder();

        #[cfg(feature = "storage")]
        if let Some(storage_folder) = options.storage_folder {
            builder = builder.with_storage_folder(&storage_folder);
        }

        if let Some(signer) = options.signer {
            builder = builder.with_signer(SignerHandle::from_str(&signer)?);
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

#[cfg(test)]
mod tests {
    use super::{AccountToCreate, MessageType, ResponseType, ManagerOptions};

    #[tokio::test]
    async fn create_account() {
        let signer = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast"}"#.to_string();
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
         }"#.to_string();

         let options = ManagerOptions {
            #[cfg(feature = "storage")]
            storage_folder: Some("teststorage".to_string()),
            client_options: Some(client_options),
            signer: Some(signer),
         };


        let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();

        // create an account
        let account = AccountToCreate { alias: None };
        let response = super::send_message(&wallet_handle, MessageType::CreateAccount(Box::new(account))).await;
        match response.response() {
            ResponseType::CreatedAccount(account) => {
                let id = account.id().clone();
                println!("Created account id: {id}")
            }
            _ => panic!("unexpected response {:?}", response),
        }
    }
}

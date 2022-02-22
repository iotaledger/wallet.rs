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

use crate::{account_manager::AccountManager, Result};
use tokio::sync::mpsc::unbounded_channel;

pub async fn create_message_handler(path: Option<String>) -> Result<WalletMessageHandler> {
    let manager = if let Some(path) = path {
        AccountManager::builder().with_storage_folder(&path).finish().await?
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
    use super::{AccountToCreate, MessageType, ResponseType};

    #[tokio::test]
    async fn create_account() {
        let wallet_handle = super::create_message_handler(None).await.unwrap();

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

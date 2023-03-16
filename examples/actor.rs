// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    actor::{AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler},
    client::ClientOptionsBuilder,
    signing::SignerType,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// The Wallet actor.
pub struct WalletActor {
    rx: UnboundedReceiver<Message>,
    message_handler: WalletMessageHandler,
}

impl WalletActor {
    /// Runs the actor.
    pub async fn run(mut self) {
        println!("running wallet actor");

        while let Some(message) = self.rx.recv().await {
            self.message_handler.handle(message).await;
        }
    }
}

fn spawn_actor() -> UnboundedSender<Message> {
    let (tx, rx) = unbounded_channel();
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async move {
            let actor = WalletActor {
                rx,
                message_handler: WalletMessageHandler::new().await.unwrap(),
            };
            actor.run().await
        });
    });
    tx
}

async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
    let (message_tx, mut message_rx) = unbounded_channel();
    let message = Message::new("".to_string(), message_type, message_tx);
    tx.send(message).unwrap();
    message_rx.recv().await.unwrap()
}

#[tokio::main]
async fn main() {
    let tx = spawn_actor();

    let account = AccountToCreate {
        client_options: ClientOptionsBuilder::new()
            .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
            .unwrap()
            .build()
            .unwrap(),
        alias: None,
        created_at: None,
        skip_persistence: false,
        signer_type: None,
        allow_create_multiple_empty_accounts: false,
    };

    send_message(&tx, MessageType::SetStrongholdPassword("password".to_string())).await;
    send_message(
        &tx,
        MessageType::StoreMnemonic {
            signer_type: SignerType::Stronghold,
            mnemonic: None,
        },
    )
    .await;
    let response = send_message(&tx, MessageType::CreateAccount(Box::new(account))).await;

    match response.response() {
        ResponseType::CreatedAccount(created_account) => {
            // remove the created account
            let response = send_message(&tx, MessageType::RemoveAccount(created_account.account.id().into())).await;
            assert!(matches!(response.response(), ResponseType::RemovedAccount(_)));
        }
        _ => panic!("unexpected response"),
    }
}

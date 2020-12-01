// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use iota_wallet::actor::{
    AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler,
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
    let actor = WalletActor {
        rx,
        message_handler: Default::default(),
    };
    std::thread::spawn(|| {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(actor.run());
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

    let account = AccountToCreate::default();
    send_message(
        &tx,
        MessageType::SetStrongholdPassword("password".to_string()),
    )
    .await;
    let response = send_message(&tx, MessageType::CreateAccount(account)).await;
    match response.response() {
        ResponseType::CreatedAccount(created_account) => {
            // remove the created account
            let response =
                send_message(&tx, MessageType::RemoveAccount(created_account.id().into())).await;
            assert!(matches!(
                response.response(),
                ResponseType::RemovedAccount(_)
            ));
        }
        _ => panic!("unexpected response"),
    }
}

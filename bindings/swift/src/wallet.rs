use iota_wallet::actor::{AccountManager, Message, MessageType, Response, WalletMessageHandler};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

/// The wallet actor builder.
#[derive(Default)]
struct WalletBuilder {
    rx: Option<UnboundedReceiver<Message>>,
    message_handler: Option<WalletMessageHandler>,
}

impl WalletBuilder {
    /// Creates a new wallet actor builder.
    fn new() -> Self {
        Self::default()
    }
    /// Sets the receiver for messages.
    fn rx(mut self, rx: UnboundedReceiver<Message>) -> Self {
        self.rx.replace(rx);
        self
    }
    /// Sets the wallet message handler
    fn message_handler(mut self, message_handler: WalletMessageHandler) -> Self {
        self.message_handler.replace(message_handler);
        self
    }
    /// Builds the Wallet actor.
    async fn build(self) -> Wallet {
        Wallet {
            rx: self.rx.expect("rx is required"),
            message_handler: self.message_handler.expect("message handler is required"),
        }
    }
}

/// The Account actor.
struct Wallet {
    rx: UnboundedReceiver<Message>,
    message_handler: WalletMessageHandler,
}

impl Wallet {
    /// Runs the actor.
    async fn run(mut self) {
        println!("running wallet actor");
        while let Some(message) = self.rx.recv().await {
            self.message_handler.handle(message).await;
        }
    }
}

pub(crate) fn runtime() -> &'static Arc<Runtime> {
    static INSTANCE: OnceCell<Arc<Runtime>> = OnceCell::new();
    let runtime = INSTANCE.get_or_init(|| Arc::new(
        Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap())
    );
    
    runtime
}

pub(crate) fn spawn_actor(manager: AccountManager) -> UnboundedSender<Message> {
    let (tx, rx) = unbounded_channel();
    std::thread::spawn(|| {
        runtime().block_on(async move {
            let actor = WalletBuilder::new()
                .rx(rx)
                .message_handler(WalletMessageHandler::with_manager(manager))
                .build()
                .await;
            actor.run().await
        });
    });
    tx
}

pub(crate) async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
    let (message_tx, mut message_rx) = unbounded_channel();
    let message = Message::new(message_type, message_tx);
    tx.send(message).unwrap();
    message_rx.recv().await.unwrap()
}

// #[tokio::test]
// async fn create_account() {
//     let manager = AccountManager::builder().finish().await.unwrap();
//     let tx = spawn_actor(manager);
//     // create an account
//     let account = AccountToCreate { alias: None };
//     let response = send_message(&tx, MessageType::CreateAccount(Box::new(account))).await;
//     match response.response() {
//         ResponseType::CreatedAccount(account) => {
//             let id = account.id().clone();
//             println!("Created account id: {id}")
//         }
//         _ => panic!("unexpected response {:?}", response),
//     }
// }

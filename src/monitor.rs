use crate::account::{Account, AccountIdentifier};
use crate::address::{Address, AddressOutput, IotaAddress};
use crate::client::ClientOptions;
use crate::message::Message;

use iota::{message::prelude::MessageId, MessageMetadata, OutputMetadata};
use once_cell::sync::Lazy;
use rumqttc::{self, AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::Deserialize;

use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

type MqttClientInstanceMap =
  Arc<Mutex<HashMap<ClientOptions, Arc<(AsyncClient, TopicHandlerMap)>>>>;
type TopicHandler =
  Box<dyn Fn(String) -> futures::future::BoxFuture<'static, crate::Result<()>> + Send>;
type TopicHandlerMap = Arc<Mutex<HashMap<String, TopicHandler>>>;

#[derive(Deserialize)]
struct AddressOutputPayload {
  #[serde(rename = "messageId")]
  message_id: String,
  #[serde(rename = "transactionId")]
  transaction_id: String,
  #[serde(rename = "outputIndex")]
  output_index: u16,
  #[serde(rename = "isSpent")]
  is_spent: bool,
  output: AddressOutputPayloadOutput,
}

#[derive(Deserialize)]
struct AddressOutputPayloadOutput {
  #[serde(rename = "type")]
  type_: u8,
  amount: u64,
  address: AddressOutputPayloadAddress,
}

#[derive(Deserialize)]
struct AddressOutputPayloadAddress {
  #[serde(rename = "type")]
  type_: u8,
  address: String,
}

/// Gets the MQTT client instances map.
fn instances() -> &'static MqttClientInstanceMap {
  static INSTANCES: Lazy<MqttClientInstanceMap> = Lazy::new(Default::default);
  &INSTANCES
}

fn get_client(client_options: &ClientOptions) -> Arc<(AsyncClient, TopicHandlerMap)> {
  let mut map = instances()
    .lock()
    .expect("failed to lock mqtt client instances: start_monitoring()");

  match map.get(&client_options) {
    Some(value) => value.clone(),
    None => {
      // TODO: pick up url from the client options
      let url = "localhost";
      let mut mqttoptions = MqttOptions::new(url, url, 1883);
      mqttoptions.set_keep_alive(5);
      let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

      let client_options_ = client_options.clone();
      thread::spawn(move || {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async move {
          loop {
            if let Ok(Event::Incoming(i)) = eventloop.poll().await {
              if let Incoming::Publish(p) = i {
                let map = instances()
                  .lock()
                  .expect("failed to lock mqtt client instances: start_monitoring() loop");
                if let Some(i) = map.get(&client_options_) {
                  let (_, topic_handlers) = i.deref();
                  let topic_handlers = topic_handlers.lock().unwrap();
                  if let Some(handler) = topic_handlers.get(&p.topic) {
                    let payload = String::from_utf8_lossy(&*p.payload).to_string();
                    println!("got topic handler with: {}", payload);
                    let _ = handler(payload).await;
                  }
                }
              }
            }
          }
        });
      });

      let value = Arc::new((client, Default::default()));
      map.insert(client_options.clone(), value.clone());
      value
    }
  }
}

fn mutate_account<F: FnOnce(&Account, &mut Vec<Address>, &mut Vec<Message>)>(
  account_id: &AccountIdentifier,
  storage_path: &PathBuf,
  cb: F,
) -> crate::Result<()> {
  let mut account = crate::storage::get_account(&storage_path, *account_id)?;
  let mut addresses: Vec<Address> = account.addresses().to_vec();
  let mut messages: Vec<Message> = account.messages().to_vec();
  cb(&account, &mut addresses, &mut messages);
  account.set_addresses(addresses);
  account.set_messages(messages);
  let account_str = serde_json::to_string(&account)?;
  crate::storage::with_adapter(&storage_path, |storage| {
    storage.set(*account_id, account_str)
  })?;
  Ok(())
}

async fn subscribe_to_topic(
  client_options: &ClientOptions,
  topic: String,
  handler: TopicHandler,
) -> crate::Result<()> {
  let i = get_client(client_options);
  let (client, topic_handlers) = i.deref();
  let mut topic_handlers = topic_handlers.lock().unwrap();
  println!("subscribing to {:?}", topic);
  client.subscribe(&topic, QoS::AtMostOnce).await?;
  topic_handlers.insert(topic, handler);
  Ok(())
}

/// Monitor account addresses for balance changes.
pub async fn monitor_account_addresses_balance(account: &Account) -> crate::Result<()> {
  for address in account.addresses().iter().filter(|a| !a.internal()) {
    monitor_address_balance(&account, &address).await?;
  }
  Ok(())
}

/// Monitor address for balance changes.
pub async fn monitor_address_balance(account: &Account, address: &Address) -> crate::Result<()> {
  let account_id_bytes = *account.id();
  let account_id: AccountIdentifier = account.id().into();
  let storage_path = account.storage_path().clone();
  let client_options = account.client_options().clone();
  let address = address.address().clone();
  let address_hex = match address {
    IotaAddress::Ed25519(ref a) => a.to_string(),
    _ => {
      return Err(crate::WalletError::GenericError(anyhow::anyhow!(
        "invalid address type"
      )))
    }
  };

  subscribe_to_topic(
    account.client_options(),
    format!("addresses/{}/outputs", address_hex),
    Box::new(move |value| {
      let address = address.clone();
      let client_options = client_options.clone();
      let storage_path = storage_path.clone();
      Box::pin(process_output(
        value,
        account_id_bytes,
        address,
        client_options,
        storage_path,
      ))
    }),
  )
  .await?;

  Ok(())
}

async fn process_output(
  value: String,
  account_id_bytes: [u8; 32],
  address: IotaAddress,
  client_options: ClientOptions,
  storage_path: PathBuf,
) -> crate::Result<()> {
  let output: AddressOutputPayload = serde_json::from_str(&value)?;
  println!("processing output");
  let account_id = account_id_bytes.into();
  let metadata = OutputMetadata {
    message_id: hex::decode(output.message_id).map_err(|e| anyhow::anyhow!(e.to_string()))?,
    transaction_id: hex::decode(output.transaction_id)
      .map_err(|e| anyhow::anyhow!(e.to_string()))?,
    output_index: output.output_index,
    is_spent: output.is_spent,
    amount: output.output.amount,
    address: address.clone(),
  };
  let address_output: AddressOutput = metadata.try_into()?;

  let client_options_ = client_options.clone();
  let message_id = address_output.message_id();
  let message_id_ = *message_id;

  let client = crate::client::get_client(&client_options_);
  let message = client.get_message().data(&message_id_).await?;

  let message_id_ = *message_id;
  mutate_account(&account_id, &storage_path, |acc, addresses, messages| {
    let address_to_update = addresses
      .iter_mut()
      .find(|a| a.address() == &address)
      .unwrap();
    address_to_update.append_output(address_output);
    crate::event::emit_balance_change(
      &account_id_bytes,
      &address_to_update,
      *address_to_update.balance(),
    );

    let message = Message::from_iota_message(message_id_, &addresses, &message).unwrap();
    if !messages.iter().any(|m| m == &message) {
      let message_id = *message.id();
      messages.push(message);
      crate::event::emit_transaction_event(
        crate::event::TransactionEventType::NewTransaction,
        &account_id_bytes,
        &message_id,
      );
    }
  })?;
  println!("done");
  Ok(())
}

/// Monitor message for confirmation state.
pub async fn monitor_confirmation_state_change(
  account: &Account,
  message_id: &MessageId,
) -> crate::Result<()> {
  let account_id_bytes = *account.id();
  let account_id: AccountIdentifier = account.id().into();
  let storage_path = account.storage_path().clone();
  let message = account
    .messages()
    .iter()
    .find(|message| message.id() == message_id)
    .unwrap()
    .clone();
  let message_id = *message_id;

  subscribe_to_topic(
    account.client_options(),
    format!("messages/{}/metadata", message_id.to_string()),
    Box::new(move |value| {
      let _ = process_metadata(value, account_id_bytes, message_id, &message, &storage_path);
      Box::pin(futures::future::ok(()))
    }),
  )
  .await?;
  Ok(())
}

fn process_metadata(
  value: String,
  account_id_bytes: [u8; 32],
  message_id: MessageId,
  message: &Message,
  storage_path: &PathBuf,
) -> crate::Result<()> {
  let account_id: AccountIdentifier = account_id_bytes.into();
  let metadata: MessageMetadata = serde_json::from_str(&value)?;
  let confirmed =
    !(metadata.should_promote.unwrap_or(true) || metadata.should_reattach.unwrap_or(true));
  if confirmed && !message.confirmed() {
    mutate_account(&account_id, &storage_path, |account, _, messages| {
      let message = messages.iter_mut().find(|m| m.id() == &message_id).unwrap();
      message.set_confirmed(true);
      crate::event::emit_confirmation_state_change(&account_id_bytes, message.id(), true);
    })?;
  }
  Ok(())
}

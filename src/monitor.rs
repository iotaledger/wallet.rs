use crate::account::{Account, AccountIdentifier};
use crate::address::{Address, AddressOutput, IotaAddress};
use crate::client::ClientOptions;
use crate::message::{Message, MessageType};

use iota::{message::prelude::MessageId, MessageMetadata, OutputMetadata, Topic, TopicEvent};
use serde::Deserialize;

use std::convert::TryInto;
use std::path::PathBuf;

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

/// Unsubscribe from all topics associated with the account.
pub fn unsubscribe(account: &Account) -> crate::Result<()> {
    let client = crate::client::get_client(account.client_options());
    let mut client = client.write().unwrap();
    client.subscriber().unsubscribe()?;
    Ok(())
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

fn subscribe_to_topic<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    client_options: &ClientOptions,
    topic: String,
    handler: C,
) -> crate::Result<()> {
    let client = crate::client::get_client(&client_options);
    let mut client = client.write().unwrap();
    client
        .subscriber()
        .topic(Topic::new(topic)?)
        .subscribe(handler)?;
    Ok(())
}

/// Monitor account addresses for balance changes.
pub fn monitor_account_addresses_balance(account: &Account) -> crate::Result<()> {
    for address in account.addresses().iter().filter(|a| !a.internal()) {
        monitor_address_balance(&account, &address)?;
    }
    Ok(())
}

/// Monitor address for balance changes.
pub fn monitor_address_balance(account: &Account, address: &Address) -> crate::Result<()> {
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
        move |topic_event| {
            let address = address.clone();
            let client_options = client_options.clone();
            let storage_path = storage_path.clone();
            crate::block_on(async {
                let _ = process_output(
                    topic_event.payload.clone(),
                    account_id_bytes,
                    address,
                    client_options,
                    storage_path,
                )
                .await;
            });
        },
    )?;

    Ok(())
}

async fn process_output(
    payload: String,
    account_id_bytes: [u8; 32],
    address: IotaAddress,
    client_options: ClientOptions,
    storage_path: PathBuf,
) -> crate::Result<()> {
    let output: AddressOutputPayload = serde_json::from_str(&payload)?;
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
    let client = client.read().unwrap();
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
    Ok(())
}

/// Monitor the account's unconfirmed messages for confirmation state change.
pub fn monitor_unconfirmed_messages(account: &Account) -> crate::Result<()> {
    for message in account.list_messages(0, 0, Some(MessageType::Unconfirmed)) {
        monitor_confirmation_state_change(&account, message.id())?;
    }
    Ok(())
}

/// Monitor message for confirmation state.
pub fn monitor_confirmation_state_change(
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
        move |value| {
            let _ = process_metadata(
                value.payload.clone(),
                account_id_bytes,
                message_id,
                &message,
                &storage_path,
            );
        },
    )?;
    Ok(())
}

fn process_metadata(
    payload: String,
    account_id_bytes: [u8; 32],
    message_id: MessageId,
    message: &Message,
    storage_path: &PathBuf,
) -> crate::Result<()> {
    let account_id: AccountIdentifier = account_id_bytes.into();
    let metadata: MessageMetadata = serde_json::from_str(&payload)?;
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

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountIdentifier},
    address::{Address, AddressOutput, IotaAddress},
    client::ClientOptions,
    message::{Message, MessageType},
};

use iota::{message::prelude::MessageId, MessageMetadata, OutputMetadata, Topic, TopicEvent};
use serde::Deserialize;

use std::{convert::TryInto, path::PathBuf};

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

fn mutate_account<F: FnOnce(&mut Account, &mut Vec<Address>, &mut Vec<Message>)>(
    account_id: &AccountIdentifier,
    storage_path: &PathBuf,
    cb: F,
) -> crate::Result<()> {
    let mut account = crate::storage::get_account(&storage_path, account_id.clone())?;
    let mut addresses: Vec<Address> = account.addresses().to_vec();
    let mut messages: Vec<Message> = account.messages().to_vec();
    cb(&mut account, &mut addresses, &mut messages);
    account.set_addresses(addresses);
    account.set_messages(messages);
    let account_str = serde_json::to_string(&account)?;
    crate::storage::with_adapter(&storage_path, |storage| storage.set(account_id.clone(), account_str))?;
    Ok(())
}

fn subscribe_to_topic<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    client_options: &ClientOptions,
    topic: String,
    handler: C,
) -> crate::Result<()> {
    let client = crate::client::get_client(&client_options);
    let mut client = client.write().unwrap();
    client.subscriber().topic(Topic::new(topic)?).subscribe(handler)?;
    Ok(())
}

/// Monitor account addresses for balance changes.
pub fn monitor_account_addresses_balance(account: &Account) -> crate::Result<()> {
    for address in account.addresses() {
        monitor_address_balance(&account, address.address())?;
    }
    Ok(())
}

/// Monitor address for balance changes.
pub fn monitor_address_balance(account: &Account, address: &IotaAddress) -> crate::Result<()> {
    let account_id_raw = account.id().clone();
    let account_id: AccountIdentifier = account.id().into();
    let storage_path = account.storage_path().clone();
    let client_options = account.client_options().clone();
    let address = address.clone();
    let address_bech32 = address.to_bech32();

    subscribe_to_topic(
        account.client_options(),
        format!("addresses/{}/outputs", address_bech32),
        move |topic_event| {
            let topic_event = topic_event.clone();
            let account_id_raw = account_id_raw.clone();
            let address = address.clone();
            let client_options = client_options.clone();
            let storage_path = storage_path.clone();
            std::thread::spawn(move || {
                crate::block_on(async {
                    let _ = process_output(
                        topic_event.payload.clone(),
                        account_id_raw.clone(),
                        address,
                        client_options,
                        storage_path,
                    )
                    .await;
                });
            });
        },
    )?;

    Ok(())
}

async fn process_output(
    payload: String,
    account_id_raw: String,
    address: IotaAddress,
    client_options: ClientOptions,
    storage_path: PathBuf,
) -> crate::Result<()> {
    let output: AddressOutputPayload = serde_json::from_str(&payload)?;
    let account_id = account_id_raw.clone().into();
    let metadata = OutputMetadata {
        message_id: hex::decode(output.message_id).map_err(|e| anyhow::anyhow!(e.to_string()))?,
        transaction_id: hex::decode(output.transaction_id).map_err(|e| anyhow::anyhow!(e.to_string()))?,
        output_index: output.output_index,
        is_spent: output.is_spent,
        amount: output.output.amount,
        address: address.clone(),
    };
    let address_output: AddressOutput = metadata.try_into()?;

    let client_options_ = client_options.clone();
    let message_id = address_output.message_id();
    let message_id_ = *message_id;

    let message = {
        let client = crate::client::get_client(&client_options_);
        let client = client.read().unwrap();
        client.get_message().data(&message_id_).await?
    };

    let message_id_ = *message_id;
    mutate_account(&account_id, &storage_path, |acc, addresses, messages| {
        let address_to_update = addresses.iter_mut().find(|a| a.address() == &address).unwrap();
        address_to_update.handle_new_output(address_output);
        crate::event::emit_balance_change(account_id_raw.clone(), &address_to_update, *address_to_update.balance());

        match messages.iter().position(|m| m.id() == &message_id_) {
            Some(message_index) => {
                let message = &mut messages[message_index];
                message.set_confirmed(true);
            }
            None => {
                let message = Message::from_iota_message(message_id_, &addresses, &message).unwrap();
                crate::event::emit_transaction_event(
                    crate::event::TransactionEventType::NewTransaction,
                    account_id_raw,
                    &message,
                );
                messages.push(message);
            }
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
pub fn monitor_confirmation_state_change(account: &Account, message_id: &MessageId) -> crate::Result<()> {
    let account_id_raw = account.id().clone();
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
        move |topic_event| {
            let _ = process_metadata(
                topic_event.payload.clone(),
                account_id_raw.clone(),
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
    account_id_raw: String,
    message_id: MessageId,
    message: &Message,
    storage_path: &PathBuf,
) -> crate::Result<()> {
    let account_id: AccountIdentifier = account_id_raw.clone().into();
    let metadata: MessageMetadata = serde_json::from_str(&payload)?;
    let confirmed = metadata.ledger_inclusion_state == Some("included".to_string());

    if confirmed != *message.confirmed() {
        mutate_account(&account_id, &storage_path, |account, addresses, messages| {
            let message = messages.iter_mut().find(|m| m.id() == &message_id).unwrap();
            message.set_confirmed(confirmed);
            if !confirmed {
                account.on_message_unconfirmed(message.id());
            }
            *addresses = account.addresses().to_vec();

            crate::event::emit_confirmation_state_change(account_id_raw, &message, confirmed);
        })?;
    }
    Ok(())
}

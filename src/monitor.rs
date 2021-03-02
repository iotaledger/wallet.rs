// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountHandle,
    address::{AddressOutput, AddressWrapper},
    client::ClientOptions,
    message::{Message, MessageType},
};

use iota::{
    bee_rest_api::handlers::{
        message_metadata::{LedgerInclusionStateDto, MessageMetadataResponse},
        output::OutputResponse,
    },
    message::prelude::MessageId,
    Topic, TopicEvent,
};

/// Unsubscribe from all topics associated with the account.
pub async fn unsubscribe(account_handle: AccountHandle) -> crate::Result<()> {
    let account = account_handle.read().await;
    let client = crate::client::get_client(account.client_options()).await;
    let mut client = client.write().await;

    let mut topics = Vec::new();
    for address in account.addresses() {
        topics.push(Topic::new(format!(
            "addresses/{}/outputs",
            address.address().to_bech32()
        ))?);
    }
    for message in account.list_messages(0, 0, Some(MessageType::Unconfirmed)) {
        topics.push(Topic::new(format!("messages/{}/metadata", message.id().to_string()))?);
    }

    client.subscriber().with_topics(topics).unsubscribe().await?;
    Ok(())
}

async fn subscribe_to_topic<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    client_options: &ClientOptions,
    topic: String,
    handler: C,
) -> crate::Result<()> {
    let client = crate::client::get_client(&client_options).await;
    let mut client = client.write().await;
    client
        .subscriber()
        .with_topic(Topic::new(topic)?)
        .subscribe(handler)
        .await?;
    Ok(())
}

/// Monitor account addresses for balance changes.
pub async fn monitor_account_addresses_balance(account_handle: AccountHandle) -> crate::Result<()> {
    let account = account_handle.read().await;
    for address in account.addresses() {
        monitor_address_balance(account_handle.clone(), address.address()).await?;
    }
    Ok(())
}

/// Monitor address for balance changes.
pub async fn monitor_address_balance(account_handle: AccountHandle, address: &AddressWrapper) -> crate::Result<()> {
    let client_options = account_handle.client_options().await;
    let client_options_ = client_options.clone();
    let address = address.clone();

    subscribe_to_topic(
        &client_options_,
        format!("addresses/{}/outputs", address.to_bech32()),
        move |topic_event| {
            log::info!("[MQTT] got {:?}", topic_event);
            let topic_event = topic_event.clone();
            let address = address.clone();
            let client_options = client_options.clone();
            let account_handle = account_handle.clone();

            crate::spawn(async move {
                if let Err(e) =
                    process_output(topic_event.payload.clone(), account_handle, address, client_options).await
                {
                    log::error!("[MQTT] error processing output: {:?}", e);
                }
            });
        },
    )
    .await
}

async fn process_output(
    payload: String,
    account_handle: AccountHandle,
    address: AddressWrapper,
    client_options: ClientOptions,
) -> crate::Result<()> {
    let output: OutputResponse = serde_json::from_str(&payload)?;

    let mut account = account_handle.write().await;
    let latest_address = account.latest_address().address().clone();
    let address_wrapper = account
        .addresses()
        .iter()
        .find(|a| a.address() == &address)
        .unwrap()
        .address()
        .clone();
    let address_output = AddressOutput::from_output_response(output, address_wrapper.bech32_hrp().to_string())?;
    let message_id = *address_output.message_id();

    let (old_balance, new_balance) = {
        let address_to_update = account
            .addresses_mut()
            .iter_mut()
            .find(|a| a.address() == &address)
            .unwrap();
        let old_balance = *address_to_update.balance();
        address_to_update.handle_new_output(address_output);
        let new_balance = *address_to_update.balance();
        (old_balance, new_balance)
    };

    let client_options_ = client_options.clone();

    crate::event::emit_balance_change(
        &account,
        &address_wrapper,
        if new_balance > old_balance {
            crate::event::BalanceChange::received(new_balance - old_balance)
        } else {
            crate::event::BalanceChange::spent(old_balance - new_balance)
        },
    )
    .await?;

    if address_wrapper == latest_address {
        account_handle.generate_address_internal(&mut account).await?;
    }

    match account.messages_mut().iter().position(|m| m.id() == &message_id) {
        Some(message_index) => {
            let message = &mut account.messages_mut()[message_index];
            message.set_confirmed(Some(true));
        }
        None => {
            if let Ok(message) = crate::client::get_client(&client_options_)
                .await
                .read()
                .await
                .get_message()
                .data(&message_id)
                .await
            {
                let message = Message::from_iota_message(message_id, message, &account)
                    .with_confirmed(Some(true))
                    .finish()
                    .await?;
                crate::event::emit_transaction_event(
                    crate::event::TransactionEventType::NewTransaction,
                    &account,
                    &message,
                )
                .await?;
                account.messages_mut().push(message);
            }
        }
    }

    account.save().await?;

    Ok(())
}

/// Monitor the account's unconfirmed messages for confirmation state change.
pub async fn monitor_unconfirmed_messages(account_handle: AccountHandle) -> crate::Result<()> {
    let account = account_handle.read().await;
    for message in account.list_messages(0, 0, Some(MessageType::Unconfirmed)) {
        monitor_confirmation_state_change(account_handle.clone(), message.id()).await?;
    }
    Ok(())
}

/// Monitor message for confirmation state.
pub async fn monitor_confirmation_state_change(
    account_handle: AccountHandle,
    message_id: &MessageId,
) -> crate::Result<()> {
    let (message, client_options) = {
        let account = account_handle.read().await;
        let message = account
            .messages()
            .iter()
            .find(|message| message.id() == message_id)
            .unwrap()
            .clone();
        (message, account.client_options().clone())
    };
    let message_id = *message_id;

    subscribe_to_topic(
        &client_options,
        format!("messages/{}/metadata", message_id.to_string()),
        move |topic_event| {
            let topic_event = topic_event.clone();
            let message = message.clone();
            let account_handle = account_handle.clone();
            crate::spawn(async move {
                if let Err(e) =
                    process_metadata(topic_event.payload.clone(), account_handle, message_id, &message).await
                {
                    log::error!("[MQTT] error processing metadata: {:?}", e);
                }
            });
        },
    )
    .await
}

async fn process_metadata(
    payload: String,
    account_handle: AccountHandle,
    message_id: MessageId,
    message: &Message,
) -> crate::Result<()> {
    let metadata: MessageMetadataResponse = serde_json::from_str(&payload)?;

    if let Some(inclusion_state) = metadata.ledger_inclusion_state {
        let confirmed = inclusion_state == LedgerInclusionStateDto::Included;
        if message.confirmed().is_none() || confirmed != message.confirmed().unwrap() {
            let mut account = account_handle.write().await;

            account
                .do_mut(|account| {
                    let messages = account.messages_mut();
                    let account_message = messages.iter_mut().find(|m| m.id() == &message_id).unwrap();
                    account_message.set_confirmed(Some(confirmed));
                    Ok(())
                })
                .await?;

            crate::event::emit_confirmation_state_change(&account, &message, confirmed).await?;
        }
    }
    Ok(())
}

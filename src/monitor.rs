// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountHandle,
    address::{AddressOutput, AddressWrapper, IotaAddress},
    client::ClientOptions,
    message::{Message, MessageType},
};

use iota::{
    bee_rest_api::types::{
        dtos::{LedgerInclusionStateDto, OutputDto},
        responses::MessageMetadataResponse,
    },
    message::prelude::MessageId,
    OutputResponse, Topic, TopicEvent,
};

use std::{
    convert::TryInto,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

/// Unsubscribe from all topics associated with the account.
pub async fn unsubscribe(account_handle: AccountHandle) -> crate::Result<()> {
    let account = account_handle.read().await;
    let client =
        crate::client::get_client(account.client_options(), Some(account_handle.is_monitoring.clone())).await?;
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

#[cfg(test)]
async fn subscribe_to_topics<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    _client_options: ClientOptions,
    _topic: Vec<Topic>,
    _is_monitoring: Arc<AtomicBool>,
    _handler: C,
) {
}

#[cfg(not(test))]
async fn subscribe_to_topics<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    client_options: ClientOptions,
    topics: Vec<Topic>,
    is_monitoring: Arc<AtomicBool>,
    handler: C,
) {
    if !topics.is_empty() {
        log::debug!("[MQTT] subscribe: {:?}", topics);
        tokio::spawn(async move {
            let client = crate::client::get_client(&client_options, Some(is_monitoring.clone())).await?;
            let mut client = client.write().await;
            if client
                .subscriber()
                .with_topics(topics)
                .subscribe(handler)
                .await
                .is_err()
            {
                is_monitoring.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            crate::Result::Ok(())
        });
    }
}

/// Monitor account addresses for balance changes.
pub async fn monitor_account_addresses_balance(account_handle: AccountHandle) {
    let account = account_handle.read().await;
    monitor_address_balance(
        account_handle.clone(),
        account.addresses().iter().map(|a| a.address().clone()).collect(),
    )
    .await;
}

/// Monitor address for balance changes.
pub async fn monitor_address_balance(account_handle: AccountHandle, addresses: Vec<AddressWrapper>) {
    let client_options = account_handle.client_options().await;
    let client_options_ = client_options.clone();

    subscribe_to_topics(
        client_options_,
        // safe to unwrap: we know the topics are valid
        addresses
            .into_iter()
            .map(|address| Topic::new(format!("addresses/{}/outputs", address.to_bech32())).unwrap())
            .collect(),
        account_handle.is_monitoring.clone(),
        move |topic_event| {
            log::info!("[MQTT] got {:?}", topic_event);
            if account_handle.is_mqtt_enabled() {
                let topic_event = topic_event.clone();
                let client_options = client_options.clone();
                let account_handle = account_handle.clone();

                crate::spawn(async move {
                    if let Err(e) = process_output(topic_event.payload.clone(), account_handle, client_options).await {
                        log::error!("[MQTT] error processing output: {:?}", e);
                    }
                });
            } else {
                log::info!("[MQTT] event ignored because account disabled mqtt");
            }
        },
    )
    .await
}

async fn process_output(
    payload: String,
    account_handle: AccountHandle,
    client_options: ClientOptions,
) -> crate::Result<()> {
    let output: OutputResponse = serde_json::from_str(&payload)?;
    let output_address: IotaAddress = match output.output.clone() {
        OutputDto::SignatureLockedSingle(output) => {
            (&output.address).try_into().map_err(|_| crate::Error::InvalidAddress)?
        }
        OutputDto::SignatureLockedDustAllowance(output) => {
            (&output.address).try_into().map_err(|_| crate::Error::InvalidAddress)?
        }
        _ => {
            log::debug!("[MQTT] ignoring output type");
            return Ok(());
        }
    };
    let mut account = account_handle.write().await;
    let address = match account
        .addresses()
        .iter()
        .find(|address| address.address().as_ref() == &output_address)
    {
        Some(address) => address.address().clone(),
        None => {
            log::debug!("[MQTT] address not found");
            return Ok(());
        }
    };

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
        let handled = address_to_update.handle_new_output(address_output)?;
        if !handled {
            // output was already handled; ignore this process
            log::debug!("[MQTT] output already handled");
            return Ok(());
        }
        let new_balance = *address_to_update.balance();
        (old_balance, new_balance)
    };

    let client_options_ = client_options.clone();

    if address_wrapper == latest_address {
        // we ignore errors in case stronghold is locked
        // because it's more important to process the event
        let _ = account_handle.generate_address_internal(&mut account).await;
    }

    match account.messages_mut().iter().position(|m| m.id() == &message_id) {
        Some(message_index) => {
            let message = &mut account.messages_mut()[message_index];
            if !message.confirmed().unwrap_or(false) {
                message.set_confirmed(Some(true));
                let message = message.clone();
                log::info!("[MQTT] message confirmed: {:?}", message.id());
                crate::event::emit_confirmation_state_change(
                    &account,
                    message.clone(),
                    true,
                    account_handle.account_options.persist_events,
                )
                .await?;
                account.save().await?;
            }
        }
        None => {
            if let Ok(message) = crate::client::get_client(&client_options_, Some(account_handle.is_monitoring.clone()))
                .await?
                .read()
                .await
                .get_message()
                .data(&message_id)
                .await
            {
                let message = Message::from_iota_message(
                    message_id,
                    message,
                    account_handle.accounts.clone(),
                    account.id(),
                    account.addresses(),
                    account.client_options(),
                )
                .with_confirmed(Some(true))
                .finish()
                .await?;
                log::info!("[MQTT] new transaction: {:?}", message.id());
                crate::event::emit_transaction_event(
                    crate::event::TransactionEventType::NewTransaction,
                    &account,
                    message.clone(),
                    account_handle.account_options.persist_events,
                )
                .await?;
                account.messages_mut().push(message);
                account.save().await?;
            }
        }
    }

    let balance_change = if new_balance > old_balance {
        crate::event::BalanceChange::received(new_balance - old_balance)
    } else {
        crate::event::BalanceChange::spent(old_balance - new_balance)
    };
    log::info!(
        "[MQTT] balance change on {} {:?}",
        address_wrapper.to_bech32(),
        balance_change
    );
    crate::event::emit_balance_change(
        &account,
        &address_wrapper,
        Some(message_id),
        balance_change,
        account_handle.account_options.persist_events,
    )
    .await?;

    account.save().await?;

    Ok(())
}

/// Monitor the account's unconfirmed messages for confirmation state change.
pub async fn monitor_unconfirmed_messages(account_handle: AccountHandle) {
    let account = account_handle.read().await;
    monitor_confirmation_state_change(
        account_handle.clone(),
        account
            .list_messages(0, 0, Some(MessageType::Unconfirmed))
            .into_iter()
            .map(|m| *m.id())
            .collect(),
    )
    .await;
}

/// Monitor message for confirmation state.
pub async fn monitor_confirmation_state_change(account_handle: AccountHandle, message_ids: Vec<MessageId>) {
    let client_options = account_handle.client_options().await;

    subscribe_to_topics(
        client_options,
        // safe to unwrap: we know the topics are valid
        message_ids
            .iter()
            .map(|id| Topic::new(format!("messages/{}/metadata", id.to_string())).unwrap())
            .collect(),
        account_handle.is_monitoring.clone(),
        move |topic_event| {
            log::info!("[MQTT] got {:?}", topic_event);
            if account_handle.is_mqtt_enabled() {
                let topic_event = topic_event.clone();
                let account_handle = account_handle.clone();
                crate::spawn(async move {
                    if let Err(e) = process_metadata(topic_event.payload.clone(), account_handle).await {
                        log::error!("[MQTT] error processing metadata: {:?}", e);
                    }
                });
            } else {
                log::info!("[MQTT] event ignored because account disabled mqtt");
            }
        },
    )
    .await
}

async fn process_metadata(payload: String, account_handle: AccountHandle) -> crate::Result<()> {
    let metadata: MessageMetadataResponse = serde_json::from_str(&payload)?;
    let message_id = MessageId::from_str(&metadata.message_id)?;

    if let Some(inclusion_state) = metadata.ledger_inclusion_state {
        let confirmed = inclusion_state == LedgerInclusionStateDto::Included;
        let mut account = account_handle.write().await;
        let messages = account.messages_mut();
        let message = messages.iter_mut().find(|m| m.id() == &message_id).unwrap();
        if message.confirmed().is_none() || confirmed != message.confirmed().unwrap() {
            let message_ = message.clone();
            message.set_confirmed(Some(confirmed));
            account.save().await?;

            log::info!(
                "[MQTT] confirmation state change: {:?} - confirmed: {}",
                message_.id(),
                confirmed
            );
            crate::event::emit_confirmation_state_change(
                &account,
                message_,
                confirmed,
                account_handle.account_options.persist_events,
            )
            .await?;
        }
    }
    Ok(())
}

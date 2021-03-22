// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountHandle, address::AddressWrapper, client::ClientOptions, message::MessageType};

use iota::{message::prelude::MessageId, Topic, TopicEvent};

use std::sync::{atomic::AtomicBool, Arc};

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
                let account_handle = account_handle.clone();
                crate::spawn(async move {
                    let _ = account_handle.sync().await.execute().await;
                });
            } else {
                log::info!("[MQTT] event ignored because account disabled mqtt");
            }
        },
    )
    .await
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
                let account_handle = account_handle.clone();
                crate::spawn(async move {
                    let _ = account_handle.sync().await.execute().await;
                });
            } else {
                log::info!("[MQTT] event ignored because account disabled mqtt");
            }
        },
    )
    .await
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{AccountHandle, AccountSynchronizeStep},
    account_manager::{AccountStore, AccountsSynchronizer},
    address::{AddressOutput, AddressWrapper, IotaAddress},
    client::ClientOptions,
    message::{Message, MessagePayload, TransactionEssence, TransactionInput, TransactionOutput},
};

use iota_client::{
    bee_rest_api::types::{dtos::OutputDto, responses::OutputResponse},
    Topic, TopicEvent,
};
use tokio::sync::RwLock;

use std::{collections::HashMap, convert::TryInto, sync::Arc};

/// Unsubscribe from all topics associated with the account.
pub async fn unsubscribe(account_handle: AccountHandle) -> crate::Result<()> {
    let account = account_handle.read().await;
    let client = crate::client::get_client(account.client_options()).await?;
    let mut client = client.write().await;

    let mut topics = Vec::new();
    for address in account.addresses() {
        topics.push(Topic::new(format!(
            "addresses/{}/outputs",
            address.address().to_bech32()
        ))?);
    }
    let message_topics = account
        .with_messages(|messages| {
            let mut topics = Vec::new();
            for m in messages {
                if m.confirmed.is_none() {
                    topics.push(Topic::new(format!("messages/{}/metadata", m.key.to_string()))?);
                }
            }
            crate::Result::Ok(topics)
        })
        .await?;
    topics.extend(message_topics);

    client.subscriber().with_topics(topics).unsubscribe().await?;
    Ok(())
}

#[cfg(test)]
async fn subscribe_to_topics<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    _client_options: ClientOptions,
    _topic: Vec<Topic>,
    _handler: C,
) {
}

#[cfg(not(test))]
async fn subscribe_to_topics<C: Fn(&TopicEvent) + Send + Sync + 'static>(
    client_options: ClientOptions,
    topics: Vec<Topic>,
    handler: C,
) {
    if !topics.is_empty() {
        log::debug!("[MQTT] subscribe: {:?}", topics);
        tokio::spawn(async move {
            let client = crate::client::get_client(&client_options).await?;
            let mut client = client.write().await;
            if let Err(err) = client.subscriber().with_topics(topics).subscribe(handler).await {
                log::debug!("[MQTT] subscribe error: {:?}", err);
            }
            crate::Result::Ok(())
        });
    }
}

/// Monitor account addresses for balance changes.
pub async fn monitor_account_addresses_balance(account_handle: AccountHandle) {
    let addresses = account_handle
        .read()
        .await
        .addresses()
        .iter()
        .map(|a| a.address().clone())
        .collect();
    monitor_address_balance(account_handle.clone(), addresses).await;
}

/// Monitor address for balance changes.
pub async fn monitor_address_balance(account_handle: AccountHandle, addresses: Vec<AddressWrapper>) {
    let client_options = account_handle.client_options().await;
    let client_options_ = client_options.clone();

    if *client_options.mqtt_enabled() {
        subscribe_to_topics(
            client_options_,
            // safe to unwrap: we know the topics are valid
            addresses
                .into_iter()
                .map(|address| Topic::new(format!("addresses/{}/outputs", address.to_bech32())).unwrap())
                .collect(),
            move |topic_event| {
                log::info!("[MQTT] got {:?}", topic_event);
                if account_handle.is_mqtt_enabled() {
                    let payload = topic_event.payload.clone();
                    let account_handle = account_handle.clone();
                    crate::spawn(async move {
                        let _ = process_output(payload, account_handle.clone()).await;
                    });
                } else {
                    log::info!("[MQTT] event ignored because account disabled mqtt");
                }
            },
        )
        .await
    }
}

async fn process_output(payload: String, account_handle: AccountHandle) -> crate::Result<bool> {
    let account = account_handle.write().await;

    let output = serde_json::from_str::<OutputResponse>(&payload)?;
    let output_address: IotaAddress = match output.output.clone() {
        OutputDto::SignatureLockedSingle(output) => {
            (&output.address).try_into().map_err(|_| crate::Error::InvalidAddress)?
        }
        OutputDto::SignatureLockedDustAllowance(output) => {
            (&output.address).try_into().map_err(|_| crate::Error::InvalidAddress)?
        }
        _ => {
            log::debug!("[MQTT] ignoring output type");
            return Ok(false);
        }
    };

    let address = match account
        .addresses()
        .iter()
        .find(|address| address.address().as_ref() == &output_address)
    {
        Some(address) => address.address().clone(),
        None => {
            log::debug!("[MQTT] address not found");
            return Ok(false);
        }
    };

    let output = AddressOutput::from_output_response(output, address.bech32_hrp().to_string())?;
    let message = match account.get_message(&output.message_id).await {
        Some(message) => {
            if !message.confirmed().unwrap_or(false) {
                message
            } else {
                return Ok(false);
            }
        }
        None => {
            if let Ok(message) = crate::client::get_client(account.client_options())
                .await?
                .read()
                .await
                .get_message()
                .data(&output.message_id)
                .await
            {
                Message::from_iota_message(
                    output.message_id,
                    message,
                    account_handle.accounts.clone(),
                    account.id(),
                    account.addresses(),
                    account.client_options(),
                )
                .with_confirmed(Some(true))
                .finish()
                .await?
            } else {
                return Ok(false);
            }
        }
    };

    let message_addresses = match message.payload() {
        Some(MessagePayload::Transaction(tx)) => match tx.essence() {
            TransactionEssence::Regular(essence) => {
                let output_addresses: Vec<AddressWrapper> = essence
                    .outputs()
                    .iter()
                    .map(|output| match output {
                        TransactionOutput::SignatureLockedDustAllowance(o) => o.address().clone(),
                        TransactionOutput::SignatureLockedSingle(o) => o.address().clone(),
                        _ => unimplemented!(),
                    })
                    .collect();
                let input_addresses: Vec<AddressWrapper> = essence
                    .inputs()
                    .iter()
                    .map(|input| match input {
                        TransactionInput::Utxo(i) => i.metadata.as_ref().map(|m| m.address.clone()),
                        _ => unimplemented!(),
                    })
                    .flatten()
                    .collect();
                let mut addresses = output_addresses;
                addresses.extend(input_addresses);
                addresses
            }
        },
        _ => vec![],
    };

    let message_is_internal = match message.payload() {
        Some(MessagePayload::Transaction(tx)) => {
            let TransactionEssence::Regular(essence) = tx.essence();
            essence.internal()
        }
        _ => false,
    };

    let storage_path = account.storage_path().clone();
    let account_id = account.id().clone();
    drop(account);

    let mut accounts_to_sync = HashMap::new();

    if message_is_internal {
        for (account_id, account_handle) in account_handle.accounts.read().await.iter() {
            if account_handle
                .read()
                .await
                .addresses()
                .iter()
                .any(|a| message_addresses.contains(a.address()))
            {
                accounts_to_sync.insert(account_id.clone(), account_handle.clone());
            }
        }
    } else {
        accounts_to_sync.insert(account_id, account_handle.clone());
    }

    // sync associated accounts
    AccountsSynchronizer::new(
        account_handle.sync_accounts_lock.clone(),
        AccountStore::new(Arc::new(RwLock::new(accounts_to_sync))),
        storage_path,
        account_handle.account_options,
    )
    .skip_account_discovery()
    .steps(vec![AccountSynchronizeStep::SyncAddresses(Some(message_addresses))])
    .execute()
    .await?;

    Ok(true)
}

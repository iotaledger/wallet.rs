// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{AccountHandle, AccountSynchronizeStep},
    address::{AddressOutput, AddressWrapper, IotaAddress},
    client::ClientOptions,
    event::{emit_confirmation_state_change, emit_transaction_event, TransactionEventType},
    message::{Message, MessagePayload, MessageType, TransactionEssence, TransactionInput, TransactionOutput},
};

use iota::{bee_rest_api::types::dtos::OutputDto, OutputResponse, Topic, TopicEvent};

use std::convert::TryInto;

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
            client.subscriber().with_topics(topics).subscribe(handler).await?;
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

async fn process_output(payload: String, account_handle: AccountHandle) -> crate::Result<bool> {
    let mut account = account_handle.write().await;

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
    let (addresses_to_sync, message_data) = if output.is_spent {
        (vec![address], None)
    } else {
        let (message, is_new) = match account.messages_mut().iter().position(|m| m.id() == &output.message_id) {
            Some(message_index) => {
                let message = &mut account.messages_mut()[message_index];
                if !message.confirmed().unwrap_or(false) {
                    message.set_confirmed(Some(true));
                    let message = message.clone();
                    (message, false)
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
                    let message = Message::from_iota_message(
                        output.message_id,
                        message,
                        account_handle.accounts.clone(),
                        account.id(),
                        account.addresses(),
                        account.client_options(),
                    )
                    .with_confirmed(Some(true))
                    .finish()
                    .await?;
                    account.messages_mut().push(message.clone());
                    (message, true)
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
                        .filter_map(|address| address)
                        .collect();
                    let mut addresses = output_addresses;
                    addresses.extend(input_addresses);
                    addresses
                }
            },
            _ => vec![],
        };
        (message_addresses, Some((message, is_new)))
    };

    drop(account);
    let _ = account_handle
        .sync()
        .await
        .steps(vec![AccountSynchronizeStep::SyncAddresses(Some(addresses_to_sync))])
        .execute()
        .await;
    let account = account_handle.read().await;

    if let Some((message, is_new)) = message_data {
        if is_new {
            emit_transaction_event(
                TransactionEventType::NewTransaction,
                &account,
                message.clone(),
                account_handle.account_options.persist_events,
            )
            .await?;
        } else {
            emit_confirmation_state_change(
                &account,
                message.clone(),
                true,
                account_handle.account_options.persist_events,
            )
            .await?;
        }
    }
    Ok(true)
}

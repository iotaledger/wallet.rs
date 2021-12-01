// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    api::finish_pow,
    bee_message::{payload::Payload, MessageId},
};

/// Submits a payload in a message
pub(crate) async fn submit_transaction_payload(
    account_handle: &AccountHandle,
    transaction_payload: TransactionPayload,
) -> crate::Result<MessageId> {
    log::debug!("[TRANSFER] send_payload");
    let account = account_handle.read().await;
    #[cfg(feature = "events")]
    let account_index = account.index;
    drop(account);
    let client = crate::client::get_client().await?;
    let local_pow = client.get_local_pow().await;
    if local_pow {
        log::debug!("[TRANSFER] doing local pow");
        #[cfg(feature = "events")]
        account_handle.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransferProgress(TransferProgressEvent::PerformingPoW),
        );
    }
    let message = finish_pow(&client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;
    // log::debug!("[TRANSFER] submitting message {:#?}", message);
    #[cfg(feature = "events")]
    account_handle.event_emitter.lock().await.emit(
        account_index,
        WalletEvent::TransferProgress(TransferProgressEvent::Broadcasting),
    );
    let message_id = client.post_message(&message).await?;
    log::debug!("[TRANSFER] submitted message {}", message_id);
    drop(client);
    // spawn a thread which tries to get the message confirmed
    tokio::spawn(async move {
        if let Ok(client) = crate::client::get_client().await {
            if let Ok(messages) = client.retry_until_included(&message_id, None, None).await {
                for msg in messages {
                    log::debug!("[TRANSFER] reattached {}, new message id {}", message_id, msg.0);
                }
            }
        }
    });
    Ok(message_id)
}

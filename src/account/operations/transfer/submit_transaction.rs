// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    api::finish_pow,
    bee_message::{payload::Payload, MessageId},
};

impl AccountHandle {
    /// Submits a payload in a message
    pub(crate) async fn submit_transaction_payload(
        &self,
        transaction_payload: TransactionPayload,
    ) -> crate::Result<MessageId> {
        log::debug!("[TRANSFER] send_payload");
        let account = self.read().await;
        #[cfg(feature = "events")]
        let account_index = account.index;
        drop(account);

        let local_pow = self.client.get_local_pow().await;
        if local_pow {
            log::debug!("[TRANSFER] doing local pow");
            #[cfg(feature = "events")]
            self.event_emitter.lock().await.emit(
                account_index,
                WalletEvent::TransferProgress(TransferProgressEvent::PerformingPoW),
            );
        }
        let message = finish_pow(&self.client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;
        // log::debug!("[TRANSFER] submitting message {:#?}", message);
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransferProgress(TransferProgressEvent::Broadcasting),
        );
        let message_id = self.client.post_message(&message).await?;
        log::debug!("[TRANSFER] submitted message {}", message_id);
        // spawn a thread which tries to get the message confirmed
        let client = self.client.clone();
        tokio::spawn(async move {
            if let Ok(messages) = client.retry_until_included(&message_id, None, None).await {
                if let Some(confirmed_message) = messages.first() {
                    if confirmed_message.0 != message_id {
                        log::debug!(
                            "[TRANSFER] reattached {}, new message id {}",
                            message_id,
                            confirmed_message.0
                        );
                    }
                }
            }
        });
        Ok(message_id)
    }
}

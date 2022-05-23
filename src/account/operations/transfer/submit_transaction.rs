// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::finish_pow,
    bee_block::{payload::Payload, BlockId},
};

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

impl AccountHandle {
    /// Submits a payload in a block
    pub(crate) async fn submit_transaction_payload(
        &self,
        transaction_payload: TransactionPayload,
    ) -> crate::Result<BlockId> {
        log::debug!("[TRANSFER] send_payload");
        let account = self.read().await;
        #[cfg(feature = "events")]
        let account_index = account.index;
        // Drop account so it's not locked during PoW
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
        let block = finish_pow(&self.client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;
        // log::debug!("[TRANSFER] submitting block {:#?}", block);
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransferProgress(TransferProgressEvent::Broadcasting),
        );
        let block_id = self.client.post_block(&block).await?;
        log::debug!("[TRANSFER] submitted block {}", block_id);
        // spawn a thread which tries to get the block confirmed
        let client = self.client.clone();
        tokio::spawn(async move {
            if let Ok(blocks) = client.retry_until_included(&block_id, None, None).await {
                if let Some(confirmed_block) = blocks.first() {
                    if confirmed_block.0 != block_id {
                        log::debug!("[TRANSFER] reattached {}, new block id {}", block_id, confirmed_block.0);
                    }
                }
            }
        });
        Ok(block_id)
    }
}

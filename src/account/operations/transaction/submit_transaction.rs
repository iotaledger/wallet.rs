// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::finish_pow,
    block::{payload::Payload, BlockId},
};

use crate::account::{handle::AccountHandle, operations::transaction::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransactionProgressEvent, WalletEvent};

impl AccountHandle {
    /// Submits a payload in a block
    pub(crate) async fn submit_transaction_payload(
        &self,
        transaction_payload: TransactionPayload,
    ) -> crate::Result<BlockId> {
        log::debug!("[TRANSACTION] send_payload");
        let account = self.read().await;
        #[cfg(feature = "events")]
        let account_index = account.index;
        // Drop account so it's not locked during PoW
        drop(account);

        let local_pow = self.client.get_local_pow().await;
        if local_pow {
            log::debug!("[TRANSACTION] doing local pow");
            #[cfg(feature = "events")]
            self.event_emitter.lock().await.emit(
                account_index,
                WalletEvent::TransactionProgress(TransactionProgressEvent::PerformingPow),
            );
        }
        let block = finish_pow(&self.client, Some(Payload::Transaction(Box::new(transaction_payload)))).await?;

        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::Broadcasting),
        );
        let block_id = self.client.post_block(&block).await?;
        log::debug!("[TRANSACTION] submitted block {}", block_id);
        Ok(block_id)
    }
}

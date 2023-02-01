// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api_types::dto::LedgerInclusionStateDto,
    block::{
        payload::{transaction::TransactionId, Payload},
        Block, BlockId,
    },
};

use crate::account::{handle::AccountHandle, types::InclusionState};

const DEFAULT_RETRY_UNTIL_INCLUDED_INTERVAL: u64 = 1;
const DEFAULT_RETRY_UNTIL_INCLUDED_MAX_AMOUNT: u64 = 40;

impl AccountHandle {
    /// Retries (promotes or reattaches) a block for provided block id until it's included (referenced by a
    /// milestone). This function is re-exported from the client library and default interval is as defined in iota.rs.
    /// Returns the included block at first position and additional reattached blocks
    pub async fn retry_until_included(
        &self,
        block_id: &BlockId,
        interval: Option<u64>,
        max_attempts: Option<u64>,
    ) -> crate::Result<Vec<(BlockId, Block)>> {
        Ok(self
            .client
            .retry_until_included(block_id, interval, max_attempts)
            .await?)
    }

    /// Retries (promotes or reattaches) a transaction sent from the account for a provided transaction id until it's
    /// included (referenced by a milestone). Returns the included block id.
    pub async fn retry_transaction_until_included(
        &self,
        transaction_id: &TransactionId,
        interval: Option<u64>,
        max_attempts: Option<u64>,
    ) -> crate::Result<BlockId> {
        log::debug!("[retry_transaction_until_included]");

        let account = self.read().await;
        let transaction = account.transactions.get(transaction_id).cloned();
        drop(account);

        if let Some(transaction) = transaction {
            if transaction.inclusion_state == InclusionState::Confirmed {
                return transaction.block_id.ok_or(crate::Error::MissingParameter("block id"));
            }

            if transaction.inclusion_state == InclusionState::Conflicting
                || transaction.inclusion_state == InclusionState::UnknownPruned
            {
                return Err(iota_client::Error::TangleInclusion(format!(
                    "transaction id: {} inclusion state: {:?}",
                    transaction_id, transaction.inclusion_state
                ))
                .into());
            }

            let block_id = match transaction.block_id {
                Some(block_id) => block_id,
                None => self
                    .client
                    .block()
                    .finish_block(Some(Payload::Transaction(Box::new(transaction.payload.clone()))))
                    .await?
                    .id(),
            };

            // Attachments of the Block to check inclusion state
            let mut block_ids = vec![block_id];
            for _ in 0..max_attempts.unwrap_or(DEFAULT_RETRY_UNTIL_INCLUDED_MAX_AMOUNT) {
                let duration =
                    std::time::Duration::from_secs(interval.unwrap_or(DEFAULT_RETRY_UNTIL_INCLUDED_INTERVAL));

                #[cfg(target_family = "wasm")]
                wasm_timer::Delay::new(duration).await?;

                #[cfg(not(target_family = "wasm"))]
                tokio::time::sleep(duration).await;

                // Check inclusion state for each attachment
                let block_ids_len = block_ids.len();
                let mut conflicting = false;
                for (index, block_id_) in block_ids.clone().iter().enumerate() {
                    let block_metadata = self.client.get_block_metadata(block_id_).await?;
                    if let Some(inclusion_state) = block_metadata.ledger_inclusion_state {
                        match inclusion_state {
                            LedgerInclusionStateDto::Included | LedgerInclusionStateDto::NoTransaction => {
                                return Ok(*block_id_);
                            }
                            // only set it as conflicting here and don't return, because another reattached block could
                            // have the included transaction
                            LedgerInclusionStateDto::Conflicting => conflicting = true,
                        };
                    }
                    // Only reattach or promote latest attachment of the block
                    if index == block_ids_len - 1 {
                        if block_metadata.should_promote.unwrap_or(false) {
                            // Safe to unwrap since we iterate over it
                            self.client.promote_unchecked(block_ids.last().unwrap()).await?;
                        } else if block_metadata.should_reattach.unwrap_or(false) {
                            let reattached_block = self
                                .client
                                .block()
                                .finish_block(Some(Payload::Transaction(Box::new(transaction.payload.clone()))))
                                .await?;
                            block_ids.push(reattached_block.id());
                        }
                    }
                }
                // After we checked all our reattached blocks, check if the transaction got reattached in another block
                // and confirmed
                if conflicting {
                    let included_block = self.client.get_included_block(transaction_id).await?;
                    return Ok(included_block.id());
                }
            }
            Err(iota_client::Error::TangleInclusion(block_id.to_string()).into())
        } else {
            Err(crate::Error::TransactionNotFound(*transaction_id))
        }
    }
}

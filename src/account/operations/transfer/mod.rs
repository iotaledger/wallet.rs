// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// transfer or transaction?

mod build_transaction;
pub(crate) mod high_level;
mod input_selection;
mod options;
mod prepare_transaction;
mod sign_transaction;
pub(crate) mod submit_transaction;

use std::time::{SystemTime, UNIX_EPOCH};

use iota_client::{
    bee_block::{
        output::{ByteCostConfig, Output},
        payload::transaction::{TransactionId, TransactionPayload},
        BlockId,
    },
    secret::types::InputSigningData,
};
use serde::Serialize;

pub use self::options::{RemainderValueStrategy, TransferOptions};
use crate::account::{
    handle::AccountHandle,
    operations::syncing::SyncOptions,
    types::{InclusionState, Transaction},
};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

/// The result of a transfer, block_id is an option because submitting the transaction could fail
#[derive(Debug, Serialize)]
pub struct TransferResult {
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    #[serde(rename = "messageId")]
    pub block_id: Option<BlockId>,
}

impl AccountHandle {
    /// Send a transaction, if sending a message fails, the function will return None for the block_id, but the wallet
    /// will retry sending the transaction during syncing.
    /// ```ignore
    /// let outputs = vec![TransferOutput {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     amount: 1_000_000,
    ///     output_kind: None,
    /// }];
    ///
    /// let res = account_handle
    ///     .send(
    ///         outputs,
    ///         Some(TransferOptions {
    ///             remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(block_id) = res.0 {
    ///     println!("Message sent: {}", block_id);
    /// }
    /// ```
    pub async fn send(&self, outputs: Vec<Output>, options: Option<TransferOptions>) -> crate::Result<TransferResult> {
        // here to check before syncing, how to prevent duplicated verification (also in send_transfer())?
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        // Check if the outputs have enough amount to cover the storage deposit
        for output in &outputs {
            output.verify_storage_deposit(&byte_cost_config)?;
        }
        // sync account before sending a transaction
        #[cfg(feature = "events")]
        {
            let account_index = self.read().await.index;
            self.event_emitter.lock().await.emit(
                account_index,
                WalletEvent::TransferProgress(TransferProgressEvent::SyncingAccount),
            );
        }
        if !options.clone().unwrap_or_default().skip_sync {
            self.sync(Some(SyncOptions {
                automatic_output_consolidation: false,
                ..Default::default()
            }))
            .await?;
        }
        self.send_transfer(outputs, options, &byte_cost_config).await
    }

    // Separated function from send, so syncing isn't called recursiv with the consolidation function, which sends
    // transfers
    pub async fn send_transfer(
        &self,
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
        byte_cost_config: &ByteCostConfig,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] send");

        let prepared_transaction_data = self.prepare_transaction(outputs, options, byte_cost_config).await?;

        let transaction_payload = match self.sign_tx_essence(&prepared_transaction_data).await {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(prepared_transaction_data.inputs_data).await?;
                return Err(err);
            }
        };

        // Ignore errors from sending, we will try to send it again during [`sync_pending_transactions`]
        let block_id = match self.submit_transaction_payload(transaction_payload.clone()).await {
            Ok(block_id) => Some(block_id),
            Err(err) => {
                log::error!("Failed to submit_transaction_payload {}", err);
                None
            }
        };

        // store transaction payload to account (with db feature also store the account to the db)
        let network_id = self.client.get_network_id().await?;
        let transaction_id = transaction_payload.id();
        let mut account = self.write().await;
        account.transactions.insert(
            transaction_id,
            Transaction {
                payload: transaction_payload,
                block_id,
                network_id,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis(),
                inclusion_state: InclusionState::Pending,
                incoming: false,
            },
        );
        account.pending_transactions.insert(transaction_id);
        #[cfg(feature = "storage")]
        {
            log::debug!("[TRANSFER] storing account {}", account.index());
            self.save(Some(&account)).await?;
        }
        Ok(TransferResult {
            transaction_id,
            block_id,
        })
    }
    // unlock outputs
    async fn unlock_inputs(&self, inputs: Vec<InputSigningData>) -> crate::Result<()> {
        let mut account = self.write().await;
        for input_signing_data in &inputs {
            account.locked_outputs.remove(&input_signing_data.output_id()?);
        }
        Ok(())
    }
}

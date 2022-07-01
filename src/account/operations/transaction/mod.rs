// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod build_transaction;
pub(crate) mod high_level;
mod input_selection;
mod options;
pub(crate) mod prepare_output;
mod prepare_transaction;
mod sign_transaction;
pub(crate) mod submit_transaction;

use std::time::{SystemTime, UNIX_EPOCH};

use iota_client::{
    api::{verify_semantic, PreparedTransactionData, SignedTransactionData},
    bee_block::{
        output::Output,
        payload::transaction::{TransactionId, TransactionPayload},
        semantic::ConflictReason,
    },
    secret::types::InputSigningData,
};
use serde::Serialize;

pub use self::options::{RemainderValueStrategy, TransactionOptions};
#[cfg(feature = "events")]
use crate::events::types::{TransactionProgressEvent, WalletEvent};
use crate::{
    account::{
        handle::AccountHandle,
        types::{InclusionState, Transaction},
    },
    iota_client::Error,
    message_interface::dtos::TransactionDto,
};

/// The result of a transaction, block_id is an option because submitting the transaction could fail
#[derive(Debug, Serialize)]
pub struct TransactionResult {
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    pub transaction: TransactionDto,
}

impl AccountHandle {
    /// Send a transaction, if sending a block fails, the function will return None for the block_id, but the wallet
    /// will retry sending the transaction during syncing.
    /// ```ignore
    /// let outputs = vec![TransactionOutput {
    ///     address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
    ///     amount: 1_000_000,
    ///     output_kind: None,
    /// }];
    ///
    /// let res = account_handle
    ///     .send(
    ///         outputs,
    ///         Some(TransactionOptions {
    ///             remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(block_id) = res.0 {
    ///     println!("Block sent: {}", block_id);
    /// }
    /// ```
    pub async fn send(
        &self,
        outputs: Vec<Output>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        // here to check before syncing, how to prevent duplicated verification (also in prepare_transaction())?
        // Checking it also here is good to return earlier if something is invalid
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
                WalletEvent::TransactionProgress(TransactionProgressEvent::SyncingAccount),
            );
        }
        if !options.clone().unwrap_or_default().skip_sync {
            self.sync(None).await?;
        }
        self.finish_transaction(outputs, options).await
    }

    /// Separated function from send, so syncing isn't called recursively with the consolidation function, which sends
    /// transactions
    pub async fn finish_transaction(
        &self,
        outputs: Vec<Output>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] finish_transaction");

        let prepared_transaction_data = self.prepare_transaction(outputs, options).await?;

        self.sign_and_submit_transaction(prepared_transaction_data).await
    }

    /// Sign a transaction, submit it to a node and store it in the account
    pub async fn sign_and_submit_transaction(
        &self,
        prepared_transaction_data: PreparedTransactionData,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] sign_and_submit_transaction");

        let signed_transaction_data = match self.sign_transaction_essence(&prepared_transaction_data).await {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(prepared_transaction_data.inputs_data).await?;
                return Err(err);
            }
        };

        self.submit_and_store_transaction(signed_transaction_data).await
    }

    /// Sync an account if not skipped in options and prepare the transaction
    pub async fn sync_and_prepare_transaction(
        &self,
        outputs: Vec<Output>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] sync_and_prepare_transaction");
        // sync account before sending a transaction
        #[cfg(feature = "events")]
        {
            let account_index = self.read().await.index;
            self.event_emitter.lock().await.emit(
                account_index,
                WalletEvent::TransactionProgress(TransactionProgressEvent::SyncingAccount),
            );
        }

        if !options.clone().unwrap_or_default().skip_sync {
            self.sync(None).await?;
        }

        self.prepare_transaction(outputs, options).await
    }

    /// Validate the transaction, submit it to a node and store it in the account
    pub async fn submit_and_store_transaction(
        &self,
        signed_transaction_data: SignedTransactionData,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] submit_and_store_transaction");

        // Validate transaction before sending and storing it
        let local_time = self.client.get_time_checked().await?;

        let conflict = verify_semantic(
            &signed_transaction_data.inputs_data,
            &signed_transaction_data.transaction_payload,
            local_time,
        )?;

        if conflict != ConflictReason::None {
            log::debug!("[TRANSACTION] conflict: {conflict:?}");
            // unlock outputs so they are available for a new transaction
            self.unlock_inputs(signed_transaction_data.inputs_data).await?;
            return Err(Error::TransactionSemantic(conflict).into());
        }

        // Ignore errors from sending, we will try to send it again during [`sync_pending_transactions`]
        let block_id = match self
            .submit_transaction_payload(signed_transaction_data.transaction_payload.clone())
            .await
        {
            Ok(block_id) => Some(block_id),
            Err(err) => {
                log::error!("Failed to submit_transaction_payload {}", err);
                None
            }
        };

        // store transaction payload to account (with db feature also store the account to the db)
        let network_id = self.client.get_network_id().await?;
        let transaction_id = signed_transaction_data.transaction_payload.id();
        let transaction = Transaction {
            payload: signed_transaction_data.transaction_payload,
            block_id,
            network_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
            inclusion_state: InclusionState::Pending,
            incoming: false,
        };
        let transaction_dto = TransactionDto::from(&transaction);
        let mut account = self.write().await;

        account.transactions.insert(transaction_id, transaction);
        account.pending_transactions.insert(transaction_id);
        #[cfg(feature = "storage")]
        {
            log::debug!("[TRANSACTION] storing account {}", account.index());
            self.save(Some(&account)).await?;
        }

        Ok(TransactionResult {
            transaction_id,
            transaction: transaction_dto,
        })
    }

    // unlock outputs
    async fn unlock_inputs(&self, inputs: Vec<InputSigningData>) -> crate::Result<()> {
        let mut account = self.write().await;
        for input_signing_data in &inputs {
            let output_id = input_signing_data.output_id()?;
            account.locked_outputs.remove(&output_id);
            log::debug!(
                "[TRANSACTION] Unlocked output {} because of transaction error",
                output_id
            );
        }
        Ok(())
    }
}

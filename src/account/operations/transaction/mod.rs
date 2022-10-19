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
    block::{output::Output, payload::transaction::TransactionPayload, semantic::ConflictReason, BlockId},
    secret::types::InputSigningData,
};

pub use self::options::{RemainderValueStrategy, TransactionOptions};
use crate::{
    account::{
        handle::AccountHandle,
        types::{InclusionState, Transaction},
    },
    iota_client::Error,
};

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
    /// let tx = account_handle
    ///     .send(
    ///         outputs,
    ///         Some(TransactionOptions {
    ///             remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// println!("Transaction created: {}", tx.transaction_id);
    /// if let Some(block_id) = tx.block_id {
    ///     println!("Block sent: {}", block_id);
    /// }
    /// ```
    pub async fn send(&self, outputs: Vec<Output>, options: Option<TransactionOptions>) -> crate::Result<Transaction> {
        // here to check before syncing, how to prevent duplicated verification (also in prepare_transaction())?
        // Checking it also here is good to return earlier if something is invalid
        let protocol_parameters = self.client.get_protocol_parameters()?;

        // Check if the outputs have enough amount to cover the storage deposit
        for output in &outputs {
            output.verify_storage_deposit(
                protocol_parameters.rent_structure().clone(),
                protocol_parameters.token_supply(),
            )?;
        }

        self.finish_transaction(outputs, options).await
    }

    /// Separated function from send, so syncing isn't called recursively with the consolidation function, which sends
    /// transactions
    pub async fn finish_transaction(
        &self,
        outputs: Vec<Output>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] finish_transaction");

        let prepared_transaction_data = self.prepare_transaction(outputs, options).await?;

        self.sign_and_submit_transaction(prepared_transaction_data).await
    }

    /// Sign a transaction, submit it to a node and store it in the account
    pub async fn sign_and_submit_transaction(
        &self,
        prepared_transaction_data: PreparedTransactionData,
    ) -> crate::Result<Transaction> {
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

    /// Validate the transaction, submit it to a node and store it in the account
    pub async fn submit_and_store_transaction(
        &self,
        signed_transaction_data: SignedTransactionData,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] submit_and_store_transaction");

        // Validate transaction before sending and storing it
        let local_time = self.client.get_time_checked()?;

        let conflict = verify_semantic(
            &signed_transaction_data.inputs_data,
            &signed_transaction_data.transaction_payload,
            local_time,
        )?;

        if conflict != ConflictReason::None {
            log::debug!(
                "[TRANSACTION] conflict: {conflict:?} for {:?}",
                signed_transaction_data.transaction_payload
            );
            // unlock outputs so they are available for a new transaction
            self.unlock_inputs(signed_transaction_data.inputs_data).await?;
            return Err(Error::TransactionSemantic(conflict).into());
        }

        // Ignore errors from sending, we will try to send it again during [`sync_pending_transactions`]
        let block_id = match self
            .submit_transaction_payload(signed_transaction_data.transaction_payload.clone())
            .await
        {
            Ok(block_id) => {
                self.monitor_tx_confirmation(block_id);
                Some(block_id)
            }
            Err(err) => {
                log::error!("Failed to submit_transaction_payload {}", err);
                None
            }
        };

        // store transaction payload to account (with db feature also store the account to the db)
        let network_id = self.client.get_network_id()?;
        let transaction_id = signed_transaction_data.transaction_payload.id();
        let transaction = Transaction {
            transaction_id: signed_transaction_data.transaction_payload.id(),
            payload: signed_transaction_data.transaction_payload,
            block_id,
            network_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_millis(),
            inclusion_state: InclusionState::Pending,
            incoming: false,
            note: None,
        };

        let mut account = self.write().await;

        account.transactions.insert(transaction_id, transaction.clone());
        account.pending_transactions.insert(transaction_id);
        #[cfg(feature = "storage")]
        {
            log::debug!("[TRANSACTION] storing account {}", account.index());
            self.save(Some(&account)).await?;
        }

        Ok(transaction)
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

    // Try to get a transaction confirmed and sync related account addresses when confirmed, so the outputs get
    // available for new transactions without manually syncing (which would sync all addresses and be more heavy without
    // extra logic)
    fn monitor_tx_confirmation(&self, block_id: BlockId) {
        // spawn a task which tries to get the block confirmed
        let account = self.clone();
        tokio::spawn(async move {
            if let Ok(blocks) = account.client().retry_until_included(&block_id, None, None).await {
                if let Some(confirmed_block) = blocks.first() {
                    if confirmed_block.0 != block_id {
                        log::debug!(
                            "[TRANSACTION] reattached {}, new block id {}",
                            block_id,
                            confirmed_block.0
                        );
                    }
                }
            }
        });
    }
}

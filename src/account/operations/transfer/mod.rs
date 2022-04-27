// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// transfer or transaction?

pub(crate) mod high_level;
mod input_selection;
mod options;
mod prepare_transaction;
mod sign_transaction;
pub(crate) mod submit_transaction;

use std::time::{SystemTime, UNIX_EPOCH};

use iota_client::{
    bee_message::{
        input::INPUT_COUNT_RANGE,
        output::{ByteCostConfig, Output, OUTPUT_COUNT_RANGE},
        payload::transaction::{TransactionId, TransactionPayload},
        MessageId,
    },
    secret::types::InputSigningData,
};
use packable::bounded::TryIntoBoundedU16Error;
use serde::Serialize;

pub use self::options::{RemainderValueStrategy, TransferOptions};
use crate::account::{
    handle::AccountHandle,
    operations::syncing::SyncOptions,
    types::{InclusionState, Transaction},
    AddressGenerationOptions,
};
#[cfg(feature = "events")]
use crate::events::types::{AddressData, TransferProgressEvent, WalletEvent};

/// The result of a transfer, message_id is an option because submitting the transaction could fail
#[derive(Debug, Serialize)]
pub struct TransferResult {
    pub transaction_id: TransactionId,
    pub message_id: Option<MessageId>,
}

impl AccountHandle {
    /// Send a transaction, if sending a message fails, the function will return None for the message_id, but the wallet
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
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
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
        // Check if the outputs have enough amount to cover the storage deposit
        for output in &outputs {
            output.verify_storage_deposit(byte_cost_config)?;
        }

        // validate amounts
        if !OUTPUT_COUNT_RANGE.contains(&(outputs.len() as u16)) {
            return Err(crate::Error::BeeMessage(
                iota_client::bee_message::Error::InvalidOutputCount(TryIntoBoundedU16Error::Truncated(outputs.len())),
            ));
        }

        let custom_inputs: Option<Vec<InputSigningData>> = {
            if let Some(options) = options.clone() {
                // validate inputs amount
                if let Some(inputs) = &options.custom_inputs {
                    if !INPUT_COUNT_RANGE.contains(&(inputs.len() as u16)) {
                        return Err(crate::Error::BeeMessage(
                            iota_client::bee_message::Error::InvalidInputCount(TryIntoBoundedU16Error::Truncated(
                                inputs.len(),
                            )),
                        ));
                    }
                    let account = self.read().await;
                    let mut input_outputs = Vec::new();
                    for output_id in inputs {
                        match account.unspent_outputs().get(output_id) {
                            Some(output) => input_outputs.push(output.input_signing_data()?),
                            None => {
                                return Err(crate::Error::CustomInputError(format!(
                                    "Custom input {} not found in unspent outputs",
                                    output_id
                                )));
                            }
                        }
                    }
                    Some(input_outputs)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let remainder_address = match &options {
            Some(options) => {
                match &options.remainder_value_strategy {
                    RemainderValueStrategy::ReuseAddress => {
                        // select_inputs will select an address from the inputs if it's none
                        None
                    }
                    RemainderValueStrategy::ChangeAddress => {
                        let remainder_address = self
                            .generate_addresses(
                                1,
                                Some(AddressGenerationOptions {
                                    internal: true,
                                    ..Default::default()
                                }),
                            )
                            .await?
                            .first()
                            .expect("Didn't generate an address")
                            .clone();
                        #[cfg(feature = "events")]
                        {
                            let account_index = self.read().await.index;
                            self.event_emitter.lock().await.emit(
                                account_index,
                                WalletEvent::TransferProgress(
                                    TransferProgressEvent::GeneratingRemainderDepositAddress(AddressData {
                                        address: remainder_address.address.to_bech32(),
                                    }),
                                ),
                            );
                        }
                        Some(remainder_address.address().inner)
                    }
                    RemainderValueStrategy::CustomAddress(address) => Some(address.address().inner),
                }
            }
            None => None,
        };

        let selected_transaction_data = self
            .select_inputs(outputs, custom_inputs, remainder_address, byte_cost_config)
            .await?;
        // can we unlock the outputs in a better way if the transaction creation fails?
        let prepared_transaction_data = match self
            .prepare_transaction(
                selected_transaction_data.inputs.clone(),
                selected_transaction_data.outputs.clone(),
                options,
            )
            .await
        {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(selected_transaction_data.inputs).await?;
                return Err(err);
            }
        };
        let transaction_payload = match self
            .sign_tx_essence(
                prepared_transaction_data.essence,
                prepared_transaction_data.input_signing_data_entries,
                selected_transaction_data.remainder_output,
            )
            .await
        {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(selected_transaction_data.inputs).await?;
                return Err(err);
            }
        };

        // Ignore errors from sending, we will try to send it again during [`sync_pending_transactions`]
        let message_id = match self.submit_transaction_payload(transaction_payload.clone()).await {
            Ok(message_id) => Some(message_id),
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
                message_id,
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
            message_id,
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

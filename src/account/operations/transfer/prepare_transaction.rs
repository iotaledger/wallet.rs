// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use iota_client::{
    api::PreparedTransactionData,
    bee_block::{
        input::INPUT_COUNT_RANGE,
        output::{ByteCostConfig, Output, OUTPUT_COUNT_RANGE},
    },
    secret::types::InputSigningData,
};
use packable::bounded::TryIntoBoundedU16Error;

use crate::account::{
    handle::AccountHandle,
    operations::transfer::{RemainderValueStrategy, TransferOptions},
    AddressGenerationOptions,
};
#[cfg(feature = "events")]
use crate::events::types::{AddressData, TransferProgressEvent, WalletEvent};

impl AccountHandle {
    /// Get inputs and build the transaction essence
    pub async fn prepare_transaction(
        &self,
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
        byte_cost_config: &ByteCostConfig,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSFER] prepare_transaction");
        let prepare_transaction_start_time = Instant::now();
        // Check if the outputs have enough amount to cover the storage deposit
        for output in &outputs {
            output.verify_storage_deposit(byte_cost_config)?;
        }

        // validate amounts
        if !OUTPUT_COUNT_RANGE.contains(&(outputs.len() as u16)) {
            return Err(crate::Error::BeeBlock(
                iota_client::bee_block::Error::InvalidOutputCount(TryIntoBoundedU16Error::Truncated(outputs.len())),
            ));
        }

        let custom_inputs: Option<Vec<InputSigningData>> = {
            if let Some(options) = options.clone() {
                // validate inputs amount
                if let Some(inputs) = &options.custom_inputs {
                    if !INPUT_COUNT_RANGE.contains(&(inputs.len() as u16)) {
                        return Err(crate::Error::BeeBlock(
                            iota_client::bee_block::Error::InvalidInputCount(TryIntoBoundedU16Error::Truncated(
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

        let prepared_transaction_data = match self
            .build_transaction_essence(selected_transaction_data.clone(), options)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(selected_transaction_data.inputs).await?;
                return Err(err);
            }
        };

        log::debug!(
            "[TRANSFER] finished prepare_transaction in {:.2?}",
            prepare_transaction_start_time.elapsed()
        );
        Ok(prepared_transaction_data)
    }
}

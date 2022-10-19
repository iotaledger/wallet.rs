// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use iota_client::{
    api::PreparedTransactionData,
    block::{
        input::INPUT_COUNT_RANGE,
        output::{Output, OUTPUT_COUNT_RANGE},
    },
    secret::types::InputSigningData,
};
use packable::bounded::TryIntoBoundedU16Error;

use crate::account::{
    handle::AccountHandle,
    operations::transaction::{RemainderValueStrategy, TransactionOptions},
};
#[cfg(feature = "events")]
use crate::events::types::{AddressData, TransactionProgressEvent, WalletEvent};

impl AccountHandle {
    /// Get inputs and build the transaction essence
    pub async fn prepare_transaction(
        &self,
        outputs: Vec<Output>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_transaction");
        let prepare_transaction_start_time = Instant::now();
        let rent_structure = self.client.get_rent_structure()?;
        let token_supply = self.client.get_token_supply()?;

        // Check if the outputs have enough amount to cover the storage deposit
        for output in &outputs {
            output.verify_storage_deposit(rent_structure.clone(), token_supply)?;
        }

        // validate amounts
        if !OUTPUT_COUNT_RANGE.contains(&(outputs.len() as u16)) {
            return Err(crate::Error::Block(iota_client::block::Error::InvalidOutputCount(
                TryIntoBoundedU16Error::Truncated(outputs.len()),
            )));
        }

        let custom_inputs: Option<Vec<InputSigningData>> = {
            if let Some(options) = &options {
                // validate inputs amount
                if let Some(inputs) = &options.custom_inputs {
                    if !INPUT_COUNT_RANGE.contains(&(inputs.len() as u16)) {
                        return Err(crate::Error::Block(iota_client::block::Error::InvalidInputCount(
                            TryIntoBoundedU16Error::Truncated(inputs.len()),
                        )));
                    }
                    let current_time = self.client.get_time_checked()?;
                    let bech32_hrp = self.client.get_bech32_hrp()?;
                    let account = self.read().await;
                    let mut input_outputs = Vec::new();
                    for output_id in inputs {
                        match account.unspent_outputs().get(output_id) {
                            Some(output) => {
                                input_outputs.push(output.input_signing_data(&account, current_time, &bech32_hrp)?)
                            }
                            None => {
                                return Err(crate::Error::CustomInputError(format!(
                                    "custom input {} not found in unspent outputs",
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
                        let remainder_address = self.generate_remainder_address().await?;
                        #[cfg(feature = "events")]
                        {
                            let account_index = self.read().await.index;
                            self.event_emitter.lock().await.emit(
                                account_index,
                                WalletEvent::TransactionProgress(
                                    TransactionProgressEvent::GeneratingRemainderDepositAddress(AddressData {
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

        let allow_burning = options.as_ref().map_or(false, |option| option.allow_burning);

        let selected_transaction_data = self
            .select_inputs(
                outputs,
                custom_inputs,
                remainder_address,
                &rent_structure,
                allow_burning,
            )
            .await?;

        let prepared_transaction_data = match self.build_transaction_essence(selected_transaction_data.clone(), options)
        {
            Ok(res) => res,
            Err(err) => {
                // unlock outputs so they are available for a new transaction
                self.unlock_inputs(selected_transaction_data.inputs).await?;
                return Err(err);
            }
        };

        log::debug!(
            "[TRANSACTION] finished prepare_transaction in {:.2?}",
            prepare_transaction_start_time.elapsed()
        );
        Ok(prepared_transaction_data)
    }
}

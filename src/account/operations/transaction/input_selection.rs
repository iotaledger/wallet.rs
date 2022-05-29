// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::input_selection::{try_select_inputs, types::SelectedTransactionData},
    bee_block::{
        address::Address,
        input::INPUT_COUNT_MAX,
        output::{ByteCostConfig, Output},
    },
    secret::types::InputSigningData,
};

use crate::account::handle::AccountHandle;
#[cfg(feature = "events")]
use crate::events::types::{TransactionProgressEvent, WalletEvent};
impl AccountHandle {
    /// Selects inputs for a transaction and locks them in the account, so they don't get used again
    pub(crate) async fn select_inputs(
        &self,
        outputs: Vec<Output>,
        custom_inputs: Option<Vec<InputSigningData>>,
        remainder_address: Option<Address>,
        byte_cost_config: &ByteCostConfig,
    ) -> crate::Result<SelectedTransactionData> {
        log::debug!("[TRANSACTION] select_inputs");
        // lock so the same inputs can't be selected in multiple transactions
        let mut account = self.write().await;
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account.index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::SelectingInputs),
        );

        // if custom inputs are provided we should only use them (validate if we have the outputs in this account and
        // that the amount is enough)
        if let Some(custom_inputs) = custom_inputs {
            // Check that no input got already locked
            for input in custom_inputs.iter() {
                if account.locked_outputs.contains(&input.output_id()?) {
                    return Err(crate::Error::CustomInputError(format!(
                        "Provided custom input {} is already used in another transaction",
                        input.output_id()?
                    )));
                }
            }

            let selected_transaction_data =
                try_select_inputs(custom_inputs, outputs, true, remainder_address, byte_cost_config, false).await?;

            // lock outputs so they don't get used by another transaction
            for output in &selected_transaction_data.inputs {
                account.locked_outputs.insert(output.output_id()?);
            }
            return Ok(selected_transaction_data);
        }

        let network_id = self.client.get_network_id().await?;

        let mut available_outputs = Vec::new();
        for (output_id, output_data) in account.unspent_outputs.iter() {
            // check if not in pending transaction (locked_outputs) and if from the correct network
            if !account.locked_outputs.contains(output_id) && output_data.network_id == network_id {
                match &output_data.output {
                    Output::Basic(basic_output) => {
                        // Only use outputs with a single unlock conditions, which is the [AddressUnlockCondition]
                        if basic_output.unlock_conditions().len() == 1 {
                            available_outputs.push(output_data.input_signing_data()?);
                        }
                    }
                    Output::Nft(nft_input) => {
                        // Only use outputs with a single unlock conditions, which is the [AddressUnlockCondition]
                        if nft_input.unlock_conditions().len() == 1 {
                            // only add if output contains same NftId
                            if outputs.iter().any(|output| {
                                if let Output::Nft(nft_output) = output {
                                    nft_input.nft_id().or_from_output_id(output_data.output_id) == *nft_output.nft_id()
                                } else {
                                    false
                                }
                            }) {
                                available_outputs.push(output_data.input_signing_data()?);
                            }
                        }
                    }
                    Output::Alias(alias_input) => {
                        // only add if output contains same AliasId
                        if outputs.iter().any(|output| {
                            if let Output::Alias(alias_output) = output {
                                alias_input.alias_id().or_from_output_id(output_data.output_id)
                                    == *alias_output.alias_id()
                            } else {
                                false
                            }
                        }) {
                            available_outputs.push(output_data.input_signing_data()?);
                        }
                    }
                    Output::Foundry(foundry_input) => {
                        // only add if output contains same FoundryId
                        if outputs.iter().any(|output| {
                            if let Output::Foundry(foundry_output) = output {
                                foundry_input.id() == foundry_output.id()
                            } else {
                                false
                            }
                        }) {
                            available_outputs.push(output_data.input_signing_data()?);
                        }
                    }
                    _ => {}
                }
            }
        }
        let selected_transaction_data = match try_select_inputs(
            available_outputs,
            outputs,
            false,
            remainder_address,
            byte_cost_config,
            false,
        )
        .await
        {
            Ok(r) => r,
            Err(iota_client::Error::ConsolidationRequired(output_count)) => {
                #[cfg(feature = "events")]
                self.event_emitter
                    .lock()
                    .await
                    .emit(account.index, WalletEvent::ConsolidationRequired);
                return Err(crate::Error::ConsolidationRequired(output_count, INPUT_COUNT_MAX));
            }
            Err(e) => return Err(e.into()),
        };

        // lock outputs so they don't get used by another transaction
        for output in &selected_transaction_data.inputs {
            log::debug!("[TRANSACTION] locking: {}", output.output_id()?);
            account.locked_outputs.insert(output.output_id()?);
        }
        Ok(selected_transaction_data)
    }
}

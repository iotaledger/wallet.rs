// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::handle::AccountHandle;
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    api::input_selection::{try_select_inputs, types::SelectedTransactionData},
    bee_message::{
        address::Address,
        input::INPUT_COUNT_MAX,
        output::{AliasId, ByteCostConfig, NftId, Output},
    },
    signing::types::InputSigningData,
};
impl AccountHandle {
    /// Selects inputs for a transaction and locks them in the account, so they don't get used again
    pub(crate) async fn select_inputs(
        &self,
        outputs: Vec<Output>,
        custom_inputs: Option<Vec<InputSigningData>>,
        remainder_address: Option<Address>,
        byte_cost_config: &ByteCostConfig,
    ) -> crate::Result<SelectedTransactionData> {
        log::debug!("[TRANSFER] select_inputs");
        // lock so the same inputs can't be selected in multiple transfers
        let mut account = self.write().await;
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account.index,
            WalletEvent::TransferProgress(TransferProgressEvent::SelectingInputs),
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
                try_select_inputs(custom_inputs, outputs, true, remainder_address, byte_cost_config).await?;

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
                            if let Some(nft_output) = outputs.iter().find(|output| {
                                if let Output::Nft(nft_output) = output {
                                    // When the nft is minted, the alias_id contains only `0` bytes and we need to
                                    // calculate the output id
                                    // todo: replace with `.or_from_output_id(output_data.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                                    let input_nft_id = if nft_input.nft_id().iter().all(|&b| b == 0) {
                                        NftId::from(&output_data.output_id)
                                    } else {
                                        *nft_input.nft_id()
                                    };
                                    input_nft_id == *nft_output.nft_id()
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
                        if let Some(alias_output) = outputs.iter().find(|output| {
                            if let Output::Alias(alias_output) = output {
                                // When the nft is minted, the alias_id contains only `0` bytes and we need to
                                // calculate the output id
                                // todo: replace with `.or_from_output_id(output_data.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                                let input_alias_id = if alias_input.alias_id().iter().all(|&b| b == 0) {
                                    AliasId::from(&output_data.output_id)
                                } else {
                                    *alias_input.alias_id()
                                };
                                input_alias_id == *alias_output.alias_id()
                            } else {
                                false
                            }
                        }) {
                            available_outputs.push(output_data.input_signing_data()?);
                        }
                    }
                    Output::Foundry(foundry_input) => {
                        // only add if output contains same FoundryId
                        if let Some(foundry_output) = outputs.iter().find(|output| {
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
        let selected_transaction_data =
            match try_select_inputs(available_outputs, outputs, false, remainder_address, byte_cost_config).await {
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
            log::debug!("[TRANSFER] locking: {}", output.output_id()?);
            account.locked_outputs.insert(output.output_id()?);
        }
        Ok(selected_transaction_data)
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{hash_map::Values, HashSet};

use iota_client::{
    api::input_selection::{Burn, InputSelection, Selected},
    block::{
        address::Address,
        input::INPUT_COUNT_MAX,
        output::{Output, OutputId},
    },
    secret::types::InputSigningData,
};

use crate::account::{
    handle::AccountHandle, operations::helpers::time::can_output_be_unlocked_forever_from_now_on, Account, OutputData,
};
#[cfg(feature = "events")]
use crate::events::types::{TransactionProgressEvent, WalletEvent};
impl AccountHandle {
    /// Selects inputs for a transaction and locks them in the account, so they don't get used again
    pub(crate) async fn select_inputs(
        &self,
        outputs: Vec<Output>,
        custom_inputs: Option<HashSet<OutputId>>,
        mandatory_inputs: Option<HashSet<OutputId>>,
        remainder_address: Option<Address>,
        burn: Option<&Burn>,
    ) -> crate::Result<Selected> {
        log::debug!("[TRANSACTION] select_inputs");
        // Voting output needs to be requested before to prevent a deadlock
        #[cfg(feature = "participation")]
        let voting_output = self.get_voting_output().await?;
        // lock so the same inputs can't be selected in multiple transactions
        let mut account = self.write().await;
        let protocol_parameters = self.client.get_protocol_parameters().await?;

        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account.index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::SelectingInputs),
        );

        let current_time = self.client.get_time_checked().await?;
        #[allow(unused_mut)]
        let mut forbidden_inputs = account.locked_outputs.clone();

        let addresses = account
            .public_addresses()
            .iter()
            .chain(account.internal_addresses().iter())
            .map(|address| *address.address.as_ref())
            .collect();

        // Prevent consuming the voting output if not actually wanted
        #[cfg(feature = "participation")]
        if let Some(voting_output) = &voting_output {
            let required = if let Some(ref mandatory_inputs) = mandatory_inputs {
                mandatory_inputs.contains(&voting_output.output_id)
            } else {
                false
            };
            if !required {
                forbidden_inputs.insert(voting_output.output_id);
            }
        }

        // Filter inputs to not include inputs that require additional outputs for storage deposit return or could be
        // still locked.
        let available_outputs_signing_data = filter_inputs(
            &account,
            account.unspent_outputs.values(),
            current_time,
            protocol_parameters.bech32_hrp(),
            &outputs,
            burn,
            custom_inputs.as_ref(),
            mandatory_inputs.as_ref(),
        )?;

        // if custom inputs are provided we should only use them (validate if we have the outputs in this account and
        // that the amount is enough)
        if let Some(custom_inputs) = custom_inputs {
            // Check that no input got already locked
            for input in custom_inputs.iter() {
                if account.locked_outputs.contains(input) {
                    return Err(crate::Error::CustomInput(format!(
                        "provided custom input {input} is already used in another transaction",
                    )));
                }
            }

            let mut input_selection = InputSelection::new(
                available_outputs_signing_data,
                outputs,
                addresses,
                protocol_parameters.clone(),
            )
            .required_inputs(custom_inputs)
            .forbidden_inputs(forbidden_inputs);

            if let Some(address) = remainder_address {
                input_selection = input_selection.remainder_address(address);
            }

            if let Some(burn) = burn {
                input_selection = input_selection.burn(burn.clone());
            }

            let selected_transaction_data = input_selection.select()?;

            // lock outputs so they don't get used by another transaction
            for output in &selected_transaction_data.inputs {
                account.locked_outputs.insert(*output.output_id());
            }

            return Ok(selected_transaction_data);
        } else if let Some(mandatory_inputs) = mandatory_inputs {
            // Check that no input got already locked
            for input in mandatory_inputs.iter() {
                if account.locked_outputs.contains(input) {
                    return Err(crate::Error::CustomInput(format!(
                        "provided custom input {input} is already used in another transaction",
                    )));
                }
            }

            let mut input_selection = InputSelection::new(
                available_outputs_signing_data,
                outputs,
                addresses,
                protocol_parameters.clone(),
            )
            .required_inputs(mandatory_inputs)
            .forbidden_inputs(forbidden_inputs);

            if let Some(address) = remainder_address {
                input_selection = input_selection.remainder_address(address);
            }

            if let Some(burn) = burn {
                input_selection = input_selection.burn(burn.clone());
            }

            let selected_transaction_data = input_selection.select()?;

            // lock outputs so they don't get used by another transaction
            for output in &selected_transaction_data.inputs {
                account.locked_outputs.insert(*output.output_id());
            }

            // lock outputs so they don't get used by another transaction
            for output in &selected_transaction_data.inputs {
                account.locked_outputs.insert(*output.output_id());
            }

            return Ok(selected_transaction_data);
        }

        let mut input_selection = InputSelection::new(
            available_outputs_signing_data,
            outputs,
            addresses,
            protocol_parameters.clone(),
        )
        .forbidden_inputs(forbidden_inputs);

        if let Some(address) = remainder_address {
            input_selection = input_selection.remainder_address(address);
        }

        if let Some(burn) = burn {
            input_selection = input_selection.burn(burn.clone());
        }

        let selected_transaction_data = match input_selection.select() {
            Ok(r) => r,
            // TODO this error doesn't exist with the new ISA
            Err(iota_client::Error::ConsolidationRequired(output_count)) => {
                #[cfg(feature = "events")]
                self.event_emitter
                    .lock()
                    .await
                    .emit(account.index, WalletEvent::ConsolidationRequired);
                return Err(crate::Error::ConsolidationRequired {
                    output_count,
                    output_count_max: INPUT_COUNT_MAX,
                });
            }
            Err(e) => return Err(e.into()),
        };

        // lock outputs so they don't get used by another transaction
        for output in &selected_transaction_data.inputs {
            log::debug!("[TRANSACTION] locking: {}", output.output_id());
            account.locked_outputs.insert(*output.output_id());
        }

        Ok(selected_transaction_data)
    }
}

/// Filter available outputs to only include outputs that don't have unlock conditions, that could create
/// conflicting transactions or need a new output for the storage deposit return
/// Also only include Alias, Nft and Foundry outputs, if a corresponding output with the same id exists in the output,
/// so they don't get burned
///
/// Note: this is only for the default input selection, it's still possible to send these outputs by using
/// `claim_outputs` or providing their OutputId's in the custom_inputs
///
/// Some examples for which outputs should be included in the inputs to select from:
/// | Unlock conditions                                   | Include in inputs |
/// | --------------------------------------------------- | ----------------- |
/// | [Address]                                           | yes               |
/// | [Address, expired Timelock]                         | yes               |
/// | [Address, not expired Timelock, ...]                | no                |
/// | [Address, expired Expiration, ...]                  | yes               |
/// | [Address, not expired Expiration, ...]              | no                |
/// | [Address, StorageDepositReturn, ...]                | no                |
/// | [Address, StorageDepositReturn, expired Expiration] | yes               |
#[allow(clippy::too_many_arguments)]
fn filter_inputs(
    account: &Account,
    available_outputs: Values<OutputId, OutputData>,
    current_time: u32,
    bech32_hrp: &str,
    outputs: &[Output],
    burn: Option<&Burn>,
    custom_inputs: Option<&HashSet<OutputId>>,
    mandatory_inputs: Option<&HashSet<OutputId>>,
) -> crate::Result<Vec<InputSigningData>> {
    let mut available_outputs_signing_data = Vec::new();

    for output_data in available_outputs {
        if !custom_inputs
            .map(|inputs| inputs.contains(&output_data.output_id))
            .unwrap_or(false)
            && !mandatory_inputs
                .map(|inputs| inputs.contains(&output_data.output_id))
                .unwrap_or(false)
        {
            let output_can_be_unlocked_now_and_in_future = can_output_be_unlocked_forever_from_now_on(
                // We use the addresses with unspent outputs, because other addresses of the
                // account without unspent outputs can't be related to this output
                &account.addresses_with_unspent_outputs,
                output_data,
                current_time,
            );

            // Outputs that could get unlocked in the future will not be included
            if !output_can_be_unlocked_now_and_in_future {
                continue;
            }
        }

        // Defaults to state transition if it is not explicitly a governance transition or a burn.
        let alias_state_transition = alias_state_transition(output_data, outputs, burn)?.unwrap_or(true);

        if let Some(available_input) =
            output_data.input_signing_data(account, current_time, bech32_hrp, alias_state_transition)?
        {
            available_outputs_signing_data.push(available_input);
        }
    }

    Ok(available_outputs_signing_data)
}

// Returns if alias transition is a state transition with the provided outputs for a given input.
pub(crate) fn alias_state_transition(
    output_data: &OutputData,
    outputs: &[Output],
    burn: Option<&Burn>,
) -> crate::Result<Option<bool>> {
    Ok(if let Output::Alias(alias_input) = &output_data.output {
        let alias_id = alias_input.alias_id_non_null(&output_data.output_id);
        // Check if alias exists in the outputs and get the required transition type
        outputs
            .iter()
            .find_map(|o| {
                if let Output::Alias(alias_output) = o {
                    if *alias_output.alias_id() == alias_id {
                        if alias_output.state_index() == alias_input.state_index() {
                            Some(Some(false))
                        } else {
                            Some(Some(true))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
                // if not find in the outputs, the alias gets burned which is a governance transaction
            })
            .unwrap_or_else(|| burn.map(|burn| !burn.aliases().contains(&alias_id)))
    } else {
        None
    })
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{hash_map::Values, HashSet};

use iota_client::{
    api::input_selection::{try_select_inputs, types::SelectedTransactionData},
    block::{
        address::Address,
        input::INPUT_COUNT_MAX,
        output::{Output, OutputId, RentStructure},
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
        custom_inputs: Option<Vec<InputSigningData>>,
        remainder_address: Option<Address>,
        rent_structure: &RentStructure,
        allow_burning: bool,
    ) -> crate::Result<SelectedTransactionData> {
        log::debug!("[TRANSACTION] select_inputs");
        // lock so the same inputs can't be selected in multiple transactions
        let mut account = self.write().await;
        let token_supply = self.client.get_token_supply()?;

        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account.index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::SelectingInputs),
        );

        let current_time = self.client.get_time_checked()?;

        // if custom inputs are provided we should only use them (validate if we have the outputs in this account and
        // that the amount is enough)
        if let Some(custom_inputs) = custom_inputs {
            // Check that no input got already locked
            for input in custom_inputs.iter() {
                if account.locked_outputs.contains(&input.output_id()?) {
                    return Err(crate::Error::CustomInputError(format!(
                        "provided custom input {} is already used in another transaction",
                        input.output_id()?
                    )));
                }
            }

            let selected_transaction_data = try_select_inputs(
                custom_inputs,
                Vec::new(),
                outputs,
                remainder_address,
                rent_structure,
                allow_burning,
                current_time,
                token_supply,
            )?;

            // lock outputs so they don't get used by another transaction
            for output in &selected_transaction_data.inputs {
                account.locked_outputs.insert(output.output_id()?);
            }

            return Ok(selected_transaction_data);
        }

        // Filter inputs to not include inputs that require additional outputs for storage deposit return or could be
        // still locked
        let bech32_hrp = self.client.get_bech32_hrp()?;
        let available_outputs_signing_data = filter_inputs(
            &account,
            account.unspent_outputs.values(),
            current_time,
            &bech32_hrp,
            &outputs,
            &account.locked_outputs,
            allow_burning,
        )?;

        let selected_transaction_data = match try_select_inputs(
            Vec::new(),
            available_outputs_signing_data,
            outputs,
            remainder_address,
            rent_structure,
            allow_burning,
            current_time,
            token_supply,
        ) {
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
fn filter_inputs(
    account: &Account,
    available_outputs: Values<OutputId, OutputData>,
    current_time: u32,
    bech32_hrp: &str,
    outputs: &[Output],
    locked_outputs: &HashSet<OutputId>,
    allow_burning: bool,
) -> crate::Result<Vec<InputSigningData>> {
    let mut available_outputs_signing_data = Vec::new();
    for output_data in available_outputs {
        // Don't use outputs that are already used in other transactions
        if locked_outputs.contains(&output_data.output_id) {
            continue;
        }

        if let Output::Foundry(foundry_input) = &output_data.output {
            // Don't add if output has not the same FoundryId, because it's the not needed unless for burning, but
            // then it should be provided in the mandatory inputs
            if !outputs.iter().any(|output| {
                if let Output::Foundry(foundry_output) = output {
                    foundry_input.id() == foundry_output.id()
                } else {
                    false
                }
            }) {
                continue;
            }
        }

        let unlock_conditions = output_data
            .output
            .unlock_conditions()
            .expect("output needs to have unlock_conditions");

        // If still time locked, don't include it
        if unlock_conditions.is_time_locked(current_time) {
            continue;
        }

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

        // If there is a StorageDepositReturnUnlockCondition and it's not expired, then don't include it
        // If the expiration is some and not expired, we would have continued already before when we check
        // output_can_be_unlocked_now_and_in_future
        if unlock_conditions.expiration().is_none() && unlock_conditions.storage_deposit_return().is_some() {
            continue;
        }

        // If alias doesn't exist in the outputs, assume the transition type that allows burning or not
        let alias_state_transition = alias_state_transition(output_data, outputs)?.unwrap_or(!allow_burning);
        available_outputs_signing_data.push(output_data.input_signing_data(
            account,
            current_time,
            bech32_hrp,
            alias_state_transition,
        )?);
    }
    Ok(available_outputs_signing_data)
}

// Returns if alias transition is a state transition with the provided outputs for a given input.
pub(crate) fn alias_state_transition(output_data: &OutputData, outputs: &[Output]) -> crate::Result<Option<bool>> {
    Ok(if let Output::Alias(alias_input) = &output_data.output {
        let alias_id = alias_input.alias_id().or_from_output_id(output_data.output_id);
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
            .unwrap_or(None)
    } else {
        None
    })
}

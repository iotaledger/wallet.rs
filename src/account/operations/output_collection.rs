// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::{
            helpers::time::{can_output_be_unlocked_now, is_expired},
            transfer::TransferResult,
        },
        types::OutputData,
        TransferOptions,
    },
    Result,
};

use iota_client::{
    api::input_selection::minimum_storage_deposit,
    bee_message::output::{
        unlock_condition::{AddressUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition},
        BasicOutputBuilder, ByteCostConfigBuilder, NativeToken, Output, OutputId,
    },
};

use std::collections::{hash_map::Entry, HashMap};

impl AccountHandle {
    /// Try to collect basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub(crate) async fn try_collect_outputs(self: &AccountHandle) -> crate::Result<Vec<TransferResult>> {
        log::debug!("[OUTPUT_COLLECTION] check if outputs can be collected");
        let account = self.read().await;
        let first_account_address = account
            .public_addresses
            .first()
            .ok_or(crate::Error::FailedToGetRemainder)?
            .clone();
        let bech32_hrp = self.client.get_bech32_hrp().await?;
        let output_consolidation_threshold = account.account_options.output_consolidation_threshold;
        let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;

        // Get outputs for the collect
        let mut outputs_to_collect: Vec<OutputData> = Vec::new();
        let mut possible_additional_inputs: Vec<OutputData> = Vec::new();
        for (output_id, output_data) in &account.unspent_outputs {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    // Only collect basic outputs
                    if let Output::Basic(basic_output) = &output.output {
                        // Ignore outputs with a single [UnlockCondition], because then it's an [AddressUnlockCondition]
                        // and we own it already without further restrictions
                        if basic_output.unlock_conditions().len() != 1
                            && can_output_be_unlocked_now(
                                &account.addresses_with_balance,
                                output,
                                local_time as u32,
                                milestone_index,
                            )
                        {
                            outputs_to_collect.push(output_data.clone());
                        } else {
                            // Store outputs with [AddressUnlockCondition] alone, because they could be used as
                            // additional input, if required
                            possible_additional_inputs.push(output_data.clone());
                        }
                    }
                }
            }
        }
        drop(account);

        if outputs_to_collect.is_empty() {
            log::debug!("[OUTPUT_COLLECTION] no outputs to collect");
            return Ok(Vec::new());
        }

        let rent_structure = self.client.get_rent_structure().await?;
        let byte_cost_config = ByteCostConfigBuilder::new()
            .byte_cost(rent_structure.v_byte_cost)
            .key_factor(rent_structure.v_byte_factor_key)
            .data_factor(rent_structure.v_byte_factor_data)
            .finish();

        let mut collection_results = Vec::new();
        // todo: remove magic number and get a value that works for the current signer (ledger is limited) and is <= max
        // inputs
        //
        // Consideration: outputs with expiration and storage deposit return unlock
        // conditions might require more outputs or maybe more additional inputs are required for the storage deposit
        // amount, that's why I set it to 5 now, because even with duplicated output amount it will be low
        // enough then
        for outputs in outputs_to_collect.chunks(5) {
            let mut outputs_to_send = Vec::new();
            // Amount we get with the storage deposit subsstracted
            let mut new_amount = 0;
            let mut new_native_tokens = HashMap::new();
            // check native tokens
            for output_data in outputs {
                // if expired we can send everything to us
                if is_expired(&output_data.output, local_time as u32, milestone_index) {
                    new_amount += output_data.output.amount();
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        for native_token in native_tokens.iter() {
                            match new_native_tokens.entry(*native_token.token_id()) {
                                Entry::Vacant(e) => {
                                    e.insert(*native_token.amount());
                                }
                                Entry::Occupied(mut e) => {
                                    *e.get_mut() += *native_token.amount();
                                }
                            }
                        }
                    }
                } else {
                    // if storage deposit return, we have to subtract this amount and create the return output
                    let mut storage_deposit = false;
                    let mut return_amount = 0;
                    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
                        if let Some(UnlockCondition::StorageDepositReturn(sdr)) =
                            unlock_conditions.get(StorageDepositReturnUnlockCondition::KIND)
                        {
                            storage_deposit = true;
                            return_amount = sdr.amount();
                            // create return output
                            outputs_to_send.push(Output::Basic(
                                BasicOutputBuilder::new(sdr.amount())?
                                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                        *sdr.return_address(),
                                    )))
                                    .finish()?,
                            ));
                        }
                    }
                    if storage_deposit {
                        // for own output subtract the return amount
                        new_amount += output_data.output.amount() - return_amount;
                        if let Some(native_tokens) = output_data.output.native_tokens() {
                            for native_token in native_tokens.iter() {
                                match new_native_tokens.entry(*native_token.token_id()) {
                                    Entry::Vacant(e) => {
                                        e.insert(*native_token.amount());
                                    }
                                    Entry::Occupied(mut e) => {
                                        *e.get_mut() += *native_token.amount();
                                    }
                                }
                            }
                        }
                    } else {
                        new_amount += output_data.output.amount();
                        if let Some(native_tokens) = output_data.output.native_tokens() {
                            for native_token in native_tokens.iter() {
                                match new_native_tokens.entry(*native_token.token_id()) {
                                    Entry::Vacant(e) => {
                                        e.insert(*native_token.amount());
                                    }
                                    Entry::Occupied(mut e) => {
                                        *e.get_mut() += *native_token.amount();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check if the new amount is enough for the storage deposit, otherwise increase it to this
            let option_native_token = if new_native_tokens.is_empty() {
                None
            } else {
                Some(new_native_tokens.clone())
            };
            let required_storage_deposit = minimum_storage_deposit(
                &byte_cost_config,
                &first_account_address.address.inner,
                &option_native_token,
            )?;
            let mut additional_inputs = Vec::new();
            if new_amount < required_storage_deposit {
                // add more inputs
                for output_data in &possible_additional_inputs {
                    // Recalculate every time, because new intputs can also add more native tokens, which would increase
                    // the storage deposit cost
                    let required_storage_deposit = minimum_storage_deposit(
                        &byte_cost_config,
                        &first_account_address.address.inner,
                        &option_native_token,
                    )?;
                    if new_amount < required_storage_deposit {
                        new_amount += output_data.output.amount();
                        if let Some(native_tokens) = output_data.output.native_tokens() {
                            for native_token in native_tokens.iter() {
                                match new_native_tokens.entry(*native_token.token_id()) {
                                    Entry::Vacant(e) => {
                                        e.insert(*native_token.amount());
                                    }
                                    Entry::Occupied(mut e) => {
                                        *e.get_mut() += *native_token.amount();
                                    }
                                }
                            }
                        }
                        additional_inputs.push(output_data.output_id);
                    } else {
                        // Break if we have enough inputs
                        break;
                    }
                }
            }

            // Create output with collected values
            outputs_to_send.push(Output::Basic(
                BasicOutputBuilder::new(new_amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        first_account_address.address.inner,
                    )))
                    .with_native_tokens(
                        new_native_tokens
                            .into_iter()
                            .map(|(id, amount)| {
                                NativeToken::new(id, amount).map_err(|e| crate::Error::ClientError(Box::new(e.into())))
                            })
                            .collect::<Result<Vec<NativeToken>>>()?,
                    )
                    .finish()?,
            ));

            match self
                .send_transfer(
                    outputs_to_send,
                    Some(TransferOptions {
                        skip_sync: true,
                        custom_inputs: Some(
                            outputs
                                .iter()
                                .map(|o| o.output_id)
                                // add additional inputs
                                .chain(additional_inputs)
                                .collect::<Vec<OutputId>>(),
                        ),
                        ..Default::default()
                    }),
                    &byte_cost_config,
                )
                .await
            {
                Ok(res) => {
                    log::debug!(
                        "[OUTPUT_COLLECTION] Collection transaction created: msg_id: {:?} tx_id: {:?}",
                        res.message_id,
                        res.transaction_id
                    );
                    collection_results.push(res);
                }
                Err(e) => log::debug!("Output collection error: {}", e),
            };
        }

        Ok(collection_results)
    }
}

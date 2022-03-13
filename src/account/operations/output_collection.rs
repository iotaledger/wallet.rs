// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::TransferResult,
        types::{address::AddressWithBalance, OutputData},
        TransferOptions,
    },
    Result,
};

use iota_client::bee_message::output::{
    unlock_condition::{
        AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
    },
    BasicOutputBuilder, ByteCostConfigBuilder, NativeToken, Output, OutputId,
};

use std::collections::{hash_map::Entry, HashMap};

impl AccountHandle {
    /// Try to collect basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub(crate) async fn collect_outputs(self: &AccountHandle) -> crate::Result<Vec<TransferResult>> {
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
        for (output_id, output_data) in &account.unspent_outputs {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    // Only collect basic outputs
                    if let Output::Basic(basic_output) = &output.output {
                        if can_output_be_unlocked_now(
                            &account.addresses_with_balance,
                            &output.output,
                            local_time as u32,
                            milestone_index,
                        ) {
                            outputs_to_collect.push(output_data.clone());
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
        // inputs Consideration for the ledger signer: outputs with expiration and storage deposit return unlock
        // conditions might require more outputs, that's why I set it to 5 now, because even with duplicated
        // output amount it will be low enough then
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
                    let mut retun_amount = 0;
                    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
                        if let Some(UnlockCondition::StorageDepositReturn(sdr)) =
                            unlock_conditions.get(StorageDepositReturnUnlockCondition::KIND)
                        {
                            storage_deposit = true;
                            retun_amount = sdr.amount();
                            // create return output
                            outputs_to_send.push(Output::Basic(
                                BasicOutputBuilder::new(retun_amount)?
                                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                        *sdr.return_address(),
                                    )))
                                    .finish()?,
                            ));
                        }
                    }
                    if storage_deposit {
                        // for own output
                        new_amount += output_data.output.amount() - retun_amount;
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
                        custom_inputs: Some(outputs.iter().map(|o| o.output_id).collect::<Vec<OutputId>>()),
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

// Check if an output can be unlocked by one of the account addresses at the current time/milestone index
fn can_output_be_unlocked_now(
    account_addresses: &[AddressWithBalance],
    output: &Output,
    current_time: u32,
    current_milestone: u32,
) -> bool {
    let mut can_be_unlocked = Vec::new();
    if let Some(unlock_conditions) = output.unlock_conditions() {
        for unlock_condition in unlock_conditions.iter() {
            match unlock_condition {
                UnlockCondition::Expiration(expiration) => {
                    let mut ms_expired = false;
                    let mut time_expired = false;
                    // 0 gets ignored
                    if *expiration.milestone_index() == 0 || *expiration.milestone_index() > current_milestone {
                        ms_expired = true;
                    }
                    if expiration.timestamp() == 0 || expiration.timestamp() > current_time {
                        time_expired = true;
                    }
                    // Check if the address which can unlock the output now is in the account
                    if ms_expired && time_expired {
                        // check return address
                        can_be_unlocked.push(
                            account_addresses
                                .iter()
                                .any(|a| a.address.inner == *expiration.return_address()),
                        );
                    } else {
                        // check address unlock condition
                        let can_unlocked = if let Some(UnlockCondition::Address(address_unlock_condition)) =
                            unlock_conditions.get(AddressUnlockCondition::KIND)
                        {
                            account_addresses
                                .iter()
                                .any(|a| a.address.inner == *address_unlock_condition.address())
                        } else {
                            false
                        };
                        can_be_unlocked.push(can_unlocked);
                    }
                }
                UnlockCondition::Timelock(timelock) => {
                    let mut ms_reached = false;
                    let mut time_reached = false;
                    // 0 gets ignored
                    if *timelock.milestone_index() == 0 || *timelock.milestone_index() > current_milestone {
                        ms_reached = true;
                    }
                    if timelock.timestamp() == 0 || timelock.timestamp() > current_time {
                        time_reached = true;
                    }
                    can_be_unlocked.push(ms_reached && time_reached);
                }
                _ => {}
            }
        }
        // If one of the unlock conditions is not met, then we can't unlock it
        !can_be_unlocked.contains(&false)
    } else {
        false
    }
}

fn is_expired(output: &Output, current_time: u32, current_milestone: u32) -> bool {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(UnlockCondition::Expiration(expiration)) = unlock_conditions.get(ExpirationUnlockCondition::KIND) {
            let mut ms_expired = false;
            let mut time_expired = false;
            // 0 gets ignored
            if *expiration.milestone_index() == 0 || *expiration.milestone_index() > current_milestone {
                ms_expired = true;
            }
            if expiration.timestamp() == 0 || expiration.timestamp() > current_time {
                time_expired = true;
            }
            // Check if the address which can unlock the output now is in the account
            ms_expired && time_expired
        } else {
            false
        }
    } else {
        false
    }
}

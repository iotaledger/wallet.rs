// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use iota_client::{
    api::input_selection::minimum_storage_deposit_basic_output,
    block::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition},
            BasicOutputBuilder, NativeTokens, NativeTokensBuilder, NftOutputBuilder, Output, OutputId,
        },
    },
};
use serde::{Deserialize, Serialize};

use crate::account::{
    handle::AccountHandle, operations::helpers::time::can_output_be_unlocked_now, types::Transaction, OutputData,
    TransactionOptions,
};

/// Enum to specify which outputs should be claimed
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OutputsToClaim {
    None = 0,
    MicroTransactions = 1,
    NativeTokens = 2,
    Nfts = 3,
    All = 4,
}

impl AccountHandle {
    /// Get basic and nft outputs that have [`ExpirationUnlockCondition`], [`StorageDepositReturnUnlockCondition`] or
    /// [`TimelockUnlockCondition`] and can be unlocked now and also get basic outputs with only an
    /// [`AddressUnlockCondition`] unlock condition, for additional inputs
    pub async fn get_unlockable_outputs_with_additional_unlock_conditions(
        &self,
        outputs_to_claim: OutputsToClaim,
    ) -> crate::Result<Vec<OutputId>> {
        log::debug!("[OUTPUT_CLAIMING] get_unlockable_outputs_with_additional_unlock_conditions");
        let account = self.read().await;

        let local_time = self.client.get_time_checked()?;

        // Get outputs for the claim
        let mut output_ids_to_claim: HashSet<OutputId> = HashSet::new();
        for (output_id, output_data) in &account.unspent_outputs {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    match &output.output {
                        Output::Basic(basic_output) => {
                            // If there is a single [UnlockCondition], then it's an
                            // [AddressUnlockCondition] and we own it already without
                            // further restrictions
                            if basic_output.unlock_conditions().len() != 1
                                && can_output_be_unlocked_now(
                                    // We use the addresses with unspent outputs, because other addresses of the
                                    // account without unspent outputs can't be related to this output
                                    &account.addresses_with_unspent_outputs,
                                    &[],
                                    output,
                                    local_time,
                                )
                            {
                                match outputs_to_claim {
                                    OutputsToClaim::MicroTransactions => {
                                        if let Some(sdr) = basic_output.unlock_conditions().storage_deposit_return() {
                                            // Only micro transaction if not the same
                                            if sdr.amount() != basic_output.amount() {
                                                output_ids_to_claim.insert(output_data.output_id);
                                            }
                                        }
                                    }
                                    OutputsToClaim::NativeTokens => {
                                        if !basic_output.native_tokens().is_empty() {
                                            output_ids_to_claim.insert(output_data.output_id);
                                        }
                                    }
                                    OutputsToClaim::All => {
                                        output_ids_to_claim.insert(output_data.output_id);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Output::Nft(nft_output) => {
                            // Ignore outputs with a single [UnlockCondition], because then it's an
                            // [AddressUnlockCondition] and we own it already without
                            // further restrictions
                            if nft_output.unlock_conditions().len() != 1
                                && can_output_be_unlocked_now(
                                    // We use the addresses with unspent outputs, because other addresses of the
                                    // account without unspent outputs can't be related to this output
                                    &account.addresses_with_unspent_outputs,
                                    &[],
                                    output,
                                    local_time,
                                )
                            {
                                match outputs_to_claim {
                                    OutputsToClaim::MicroTransactions => {
                                        if let Some(sdr) = nft_output.unlock_conditions().storage_deposit_return() {
                                            // Only micro transaction if not the same
                                            if sdr.amount() != nft_output.amount() {
                                                output_ids_to_claim.insert(output_data.output_id);
                                            }
                                        }
                                    }
                                    OutputsToClaim::NativeTokens => {
                                        if !nft_output.native_tokens().is_empty() {
                                            output_ids_to_claim.insert(output_data.output_id);
                                        }
                                    }
                                    OutputsToClaim::Nfts | OutputsToClaim::All => {
                                        output_ids_to_claim.insert(output_data.output_id);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // Other output types can't have [`ExpirationUnlockCondition`],
                        // [`StorageDepositReturnUnlockCondition`] or [`TimelockUnlockCondition`]
                        _ => {}
                    }
                }
            }
        }
        log::debug!(
            "[OUTPUT_CLAIMING] available outputs to claim: {}",
            output_ids_to_claim.len()
        );
        Ok(output_ids_to_claim.into_iter().collect())
    }

    /// Get basic outputs that have only one unlock condition which is [AddressUnlockCondition], so they can be used as
    /// additional inputs
    pub async fn get_basic_outputs_for_additional_inputs(&self) -> crate::Result<Vec<OutputData>> {
        log::debug!("[OUTPUT_CLAIMING] get_basic_outputs_for_additional_inputs");
        let account = self.read().await;

        // Get basic outputs only with AddressUnlockCondition and no other unlock condition
        let mut basic_outputs: Vec<OutputData> = Vec::new();
        for (output_id, output_data) in &account.unspent_outputs {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    if let Output::Basic(basic_output) = &output.output {
                        if basic_output.unlock_conditions().len() == 1 {
                            // Store outputs with [`AddressUnlockCondition`] alone, because they could be used as
                            // additional input, if required
                            basic_outputs.push(output_data.clone());
                        }
                    }
                }
            }
        }
        log::debug!("[OUTPUT_CLAIMING] available basic outputs: {}", basic_outputs.len());
        Ok(basic_outputs)
    }

    /// Try to claim basic or nft outputs that have additional unlock conditions to their [AddressUnlockCondition]
    /// from [`AccountHandle::get_unlockable_outputs_with_additional_unlock_conditions()`].
    pub async fn claim_outputs(&self, output_ids_to_claim: Vec<OutputId>) -> crate::Result<Transaction> {
        log::debug!("[OUTPUT_CLAIMING] claim_outputs");
        let basic_outputs = self.get_basic_outputs_for_additional_inputs().await?;
        self.claim_outputs_internal(output_ids_to_claim, basic_outputs).await
    }

    /// Try to claim basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub(crate) async fn claim_outputs_internal(
        &self,
        output_ids_to_claim: Vec<OutputId>,
        mut possible_additional_inputs: Vec<OutputData>,
    ) -> crate::Result<Transaction> {
        log::debug!("[OUTPUT_CLAIMING] claim_outputs_internal");

        let current_time = self.client.get_time_checked()?;
        let rent_structure = self.client.get_rent_structure()?;
        let token_supply = self.client.get_token_supply()?;

        let account = self.read().await;

        let mut outputs_to_claim = Vec::new();
        for output_id in output_ids_to_claim {
            if let Some(output_data) = account.unspent_outputs.get(&output_id) {
                if !account.locked_outputs.contains(&output_id) {
                    outputs_to_claim.push(output_data.clone());
                }
            }
        }

        if outputs_to_claim.is_empty() {
            return Err(crate::Error::CustomInputError(
                "provided outputs can't be claimed".to_string(),
            ));
        }

        let first_account_address = account
            .public_addresses
            .first()
            .ok_or(crate::Error::FailedToGetRemainder)?
            .clone();
        drop(account);

        let mut additional_inputs_used = HashSet::new();

        // Outputs with expiration and storage deposit return might require two outputs if there is a storage deposit
        // return unlock condition Maybe also more additional inputs are required for the storage deposit, if we
        // have to send the storage deposit back.

        let mut outputs_to_send = Vec::new();
        // Keep track of the outputs to return, so we only create one output per address
        let mut required_address_returns: HashMap<Address, u64> = HashMap::new();
        // Amount we get with the storage deposit return amounts already subtracted
        let mut available_amount = 0;
        let mut required_amount_for_nfts = 0;
        let mut new_native_tokens = NativeTokensBuilder::new();
        // check native tokens
        for output_data in &outputs_to_claim {
            if let Some(native_tokens) = output_data.output.native_tokens() {
                // Skip output if the max native tokens count would be exceeded
                if get_new_native_token_count(&new_native_tokens, native_tokens)? > NativeTokens::COUNT_MAX.into() {
                    log::debug!("[OUTPUT_CLAIMING] skipping output to not exceed the max native tokens count");
                    continue;
                }
                new_native_tokens.add_native_tokens(native_tokens.clone())?;
            }
            if let Some(sdr) = sdr_not_expired(&output_data.output, current_time) {
                // for own output subtract the return amount
                available_amount += output_data.output.amount() - sdr.amount();

                // Insert for return output
                *required_address_returns.entry(*sdr.return_address()).or_default() += sdr.amount();
            } else {
                available_amount += output_data.output.amount();
            }

            if let Output::Nft(nft_output) = &output_data.output {
                // build new output with same amount, nft_id, immutable/feature blocks and native tokens, just
                // updated address unlock conditions

                let nft_output = NftOutputBuilder::from(nft_output)
                    .with_minimum_storage_deposit(rent_structure.clone())
                    .with_nft_id(nft_output.nft_id().or_from_output_id(output_data.output_id))
                    .with_unlock_conditions([UnlockCondition::Address(AddressUnlockCondition::new(
                        first_account_address.address.inner,
                    ))])
                    // Set native tokens empty, we will collect them from all inputs later
                    .with_native_tokens([])
                    .finish_output(token_supply)?;

                // Add required amount for the new output
                required_amount_for_nfts += nft_output.amount();
                outputs_to_send.push(nft_output);
            }
        }

        let option_native_token = if new_native_tokens.is_empty() {
            None
        } else {
            Some(new_native_tokens.clone().finish()?)
        };

        // Check if the new amount is enough for the storage deposit, otherwise increase it to this
        let mut required_amount = required_amount_for_nfts
            + minimum_storage_deposit_basic_output(
                &rent_structure,
                &first_account_address.address.inner,
                &option_native_token,
                token_supply,
            )?;

        let mut additional_inputs = Vec::new();
        if available_amount < required_amount {
            // Sort by amount so we use as less as possible
            possible_additional_inputs.sort_by_key(|o| o.output.amount());

            // add more inputs
            for output_data in &possible_additional_inputs {
                let option_native_token = if new_native_tokens.is_empty() {
                    None
                } else {
                    Some(new_native_tokens.clone().finish()?)
                };
                // Recalculate every time, because new inputs can also add more native tokens, which would increase
                // the required storage deposit
                required_amount = required_amount_for_nfts
                    + minimum_storage_deposit_basic_output(
                        &rent_structure,
                        &first_account_address.address.inner,
                        &option_native_token,
                        token_supply,
                    )?;
                if available_amount < required_amount {
                    if !additional_inputs_used.contains(&output_data.output_id) {
                        if let Some(native_tokens) = output_data.output.native_tokens() {
                            // Skip input if the max native tokens count would be exceeded
                            if get_new_native_token_count(&new_native_tokens, native_tokens)?
                                > NativeTokens::COUNT_MAX.into()
                            {
                                log::debug!(
                                    "[OUTPUT_CLAIMING] skipping input to not exceed the max native tokens count"
                                );
                                continue;
                            }
                            new_native_tokens.add_native_tokens(native_tokens.clone())?;
                        }
                        available_amount += output_data.output.amount();
                        additional_inputs.push(output_data.output_id);
                        additional_inputs_used.insert(output_data.output_id);
                    }
                } else {
                    // Break if we have enough inputs
                    break;
                }
            }
        }

        // If we still don't have enough amount we can't create the output
        if available_amount < required_amount {
            return Err(crate::Error::InsufficientFunds(available_amount, required_amount));
        }

        for (return_address, return_amount) in required_address_returns {
            outputs_to_send.push(
                BasicOutputBuilder::new_with_amount(return_amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(return_address)))
                    .finish_output(token_supply)?,
            );
        }

        // Create output with claimed values
        outputs_to_send.push(
            BasicOutputBuilder::new_with_amount(available_amount - required_amount_for_nfts)?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    first_account_address.address.inner,
                )))
                .with_native_tokens(new_native_tokens.finish()?)
                .finish_output(token_supply)?,
        );

        let claim_tx = self
            .finish_transaction(
                outputs_to_send,
                Some(TransactionOptions {
                    custom_inputs: Some(
                        outputs_to_claim
                            .iter()
                            .map(|o| o.output_id)
                            // add additional inputs
                            .chain(additional_inputs)
                            .collect::<Vec<OutputId>>(),
                    ),
                    ..Default::default()
                }),
            )
            .await?;

        log::debug!(
            "[OUTPUT_CLAIMING] Claiming transaction created: block_id: {:?} tx_id: {:?}",
            claim_tx.block_id,
            claim_tx.transaction_id
        );
        Ok(claim_tx)
    }
}

/// Get the `StorageDepositReturnUnlockCondition`, if not expired
pub(crate) fn sdr_not_expired(output: &Output, current_time: u32) -> Option<&StorageDepositReturnUnlockCondition> {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(sdr) = unlock_conditions.storage_deposit_return() {
            let expired = if let Some(expiration) = unlock_conditions.expiration() {
                current_time >= expiration.timestamp()
            } else {
                false
            };

            // We only have to send the storage deposit return back if the output is not expired
            if !expired { Some(sdr) } else { None }
        } else {
            None
        }
    } else {
        None
    }
}

// Helper function to calculate the native token count without duplicates, when new native tokens are added
// Might be possible to refactor the sections where it's used to remove the clones
pub(crate) fn get_new_native_token_count(
    native_tokens_builder: &NativeTokensBuilder,
    native_tokens: &NativeTokens,
) -> crate::Result<usize> {
    // Clone to get the new native token count without actually modifying it
    let mut native_tokens_count = native_tokens_builder.clone();
    native_tokens_count.add_native_tokens(native_tokens.clone())?;
    Ok(native_tokens_count.len())
}

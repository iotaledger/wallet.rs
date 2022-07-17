// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use iota_client::{
    api::input_selection::minimum_storage_deposit,
    block::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition},
            BasicOutputBuilder, NativeTokensBuilder, NftOutputBuilder, Output, OutputId,
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

/// Defines how many inputs can be claimed with one transaction.
/// This limit is needed because in addition we might need to create the double amount of outputs because of the storage
/// deposit return and also consider the remainder output.
const MAX_CLAIM_INPUTS: usize = 60;

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

        let local_time = self.client.get_time_checked().await?;

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

    /// Try to claim basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub async fn try_claim_outputs(&self, outputs_to_claim: OutputsToClaim) -> crate::Result<Vec<Transaction>> {
        log::debug!("[OUTPUT_CLAIMING] try_claim_outputs");

        let output_ids_to_claim = self
            .get_unlockable_outputs_with_additional_unlock_conditions(outputs_to_claim)
            .await?;
        let basic_outputs = self.get_basic_outputs_for_additional_inputs().await?;
        self.claim_outputs_internal(output_ids_to_claim, basic_outputs).await
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
    pub async fn claim_outputs(&self, output_ids_to_claim: Vec<OutputId>) -> crate::Result<Vec<Transaction>> {
        log::debug!("[OUTPUT_CLAIMING] claim_outputs");
        let basic_outputs = self.get_basic_outputs_for_additional_inputs().await?;
        self.claim_outputs_internal(output_ids_to_claim, basic_outputs).await
    }

    /// Try to claim basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub(crate) async fn claim_outputs_internal(
        &self,
        output_ids_to_claim: Vec<OutputId>,
        possible_additional_inputs: Vec<OutputData>,
    ) -> crate::Result<Vec<Transaction>> {
        log::debug!("[OUTPUT_CLAIMING] claim_outputs_internal");

        let current_time = self.client.get_time_checked().await?;
        let byte_cost_config = self.client.get_byte_cost_config().await?;

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
            // No outputs to claim, return
            return Ok(Vec::new());
        }

        let first_account_address = account
            .public_addresses
            .first()
            .ok_or(crate::Error::FailedToGetRemainder)?
            .clone();
        drop(account);

        let mut claim_results = Vec::new();
        let mut additional_inputs_used = HashSet::new();

        // Outputs with expiration and storage deposit return might require two outputs if there is a storage deposit
        // return unlock condition Maybe also more additional inputs are required for the storage deposit, if we
        // have to send the storage deposit back.
        for outputs in outputs_to_claim.chunks(MAX_CLAIM_INPUTS) {
            let mut outputs_to_send = Vec::new();
            // Keep track of the outputs to return, so we only create one output per address
            let mut required_address_returns: HashMap<Address, u64> = HashMap::new();
            // Amount we get with the storage deposit return amounts already subtracted
            let mut new_amount = 0;
            let mut new_native_tokens = NativeTokensBuilder::new();
            // check native tokens
            for output_data in outputs {
                if let Output::Nft(nft_output) = &output_data.output {
                    // build new output with same amount, nft_id, immutable/feature blocks and native tokens, just
                    // updated address unlock conditions

                    let nft_output = NftOutputBuilder::from(nft_output)
                        .with_minimum_storage_deposit(byte_cost_config.clone())
                        .with_nft_id(nft_output.nft_id().or_from_output_id(output_data.output_id))
                        .with_unlock_conditions([UnlockCondition::Address(AddressUnlockCondition::new(
                            first_account_address.address.inner,
                        ))])
                        // Set native tokens empty, we will collect them from all inputs later
                        .with_native_tokens([])
                        .finish_output()?;

                    outputs_to_send.push(nft_output);
                }

                if let Some(sdr) = sdr_not_expired(&output_data.output, current_time) {
                    // for own output subtract the return amount
                    new_amount += output_data.output.amount() - sdr.amount();
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        new_native_tokens.add_native_tokens(native_tokens.clone())?;
                    }

                    // Insert for return output
                    *required_address_returns.entry(*sdr.return_address()).or_default() += sdr.amount();
                } else {
                    new_amount += output_data.output.amount();
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        new_native_tokens.add_native_tokens(native_tokens.clone())?;
                    }
                }
            }

            let option_native_token = if new_native_tokens.is_empty() {
                None
            } else {
                Some(new_native_tokens.clone().finish()?)
            };

            // Check if the new amount is enough for the storage deposit, otherwise increase it to this
            let mut required_storage_deposit = minimum_storage_deposit(
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
                    required_storage_deposit = minimum_storage_deposit(
                        &byte_cost_config,
                        &first_account_address.address.inner,
                        &option_native_token,
                    )?;
                    if new_amount < required_storage_deposit {
                        if !additional_inputs_used.contains(&output_data.output_id) {
                            new_amount += output_data.output.amount();
                            if let Some(native_tokens) = output_data.output.native_tokens() {
                                new_native_tokens.add_native_tokens(native_tokens.clone())?;
                            }
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
            if new_amount < required_storage_deposit {
                return Err(crate::Error::InsufficientFunds(new_amount, required_storage_deposit));
            }

            for (return_address, return_amount) in required_address_returns {
                outputs_to_send.push(
                    BasicOutputBuilder::new_with_amount(return_amount)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(return_address)))
                        .finish_output()?,
                );
            }

            // Create output with claimed values
            outputs_to_send.push(
                BasicOutputBuilder::new_with_amount(new_amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        first_account_address.address.inner,
                    )))
                    .with_native_tokens(new_native_tokens.finish()?)
                    .finish_output()?,
            );

            match self
                .finish_transaction(
                    outputs_to_send,
                    Some(TransactionOptions {
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
                )
                .await
            {
                Ok(tx) => {
                    log::debug!(
                        "[OUTPUT_CLAIMING] Claiming transaction created: block_id: {:?} tx_id: {:?}",
                        tx.block_id,
                        tx.transaction_id
                    );
                    claim_results.push(tx);
                }
                Err(e) => log::debug!("Output claim error: {}", e),
            };
        }

        Ok(claim_results)
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

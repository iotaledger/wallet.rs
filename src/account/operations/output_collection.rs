// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::{
            helpers::time::{can_output_be_unlocked_now, is_expired},
            transfer::TransferResult,
        },
        OutputData, TransferOptions,
    },
    Result,
};

use iota_client::{
    api::input_selection::minimum_storage_deposit,
    bee_message::output::{
        unlock_condition::{AddressUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition},
        BasicOutputBuilder, NativeToken, NftOutputBuilder, Output, OutputId,
    },
};

use serde::{Deserialize, Serialize};

use std::collections::{hash_map::Entry, HashMap, HashSet};

/// Enum to specify which outputs should be collected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputsToCollect {
    None = 0,
    MicroTransactions = 1,
    NativeTokens = 2,
    Nfts = 3,
    All = 4,
}

impl AccountHandle {
    /// Get basic and nft outputs that have more than the [`AddressUnlockCondition`] and also get basic outputs with
    /// only this unlock condition, for additional inputs
    pub async fn get_outputs_with_additional_unlock_conditions(
        &self,
        outputs_to_collect: OutputsToCollect,
    ) -> crate::Result<Vec<OutputId>> {
        log::debug!("[OUTPUT_COLLECTION] get_outputs_with_additional_unlock_conditions");
        let account = self.read().await;

        let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;

        // Get outputs for the collect
        let mut output_ids_to_collect: HashSet<OutputId> = HashSet::new();
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
                                    &account.addresses_with_balance,
                                    output,
                                    local_time as u32,
                                    milestone_index,
                                )
                            {
                                match outputs_to_collect {
                                    OutputsToCollect::MicroTransactions => {
                                        if let Some(UnlockCondition::StorageDepositReturn(sdr)) = basic_output
                                            .unlock_conditions()
                                            .get(StorageDepositReturnUnlockCondition::KIND)
                                        {
                                            // Only micro transaction if not the same
                                            if sdr.amount() != basic_output.amount() {
                                                output_ids_to_collect.insert(output_data.output_id);
                                            }
                                        }
                                    }
                                    OutputsToCollect::NativeTokens => {
                                        if !basic_output.native_tokens().is_empty() {
                                            output_ids_to_collect.insert(output_data.output_id);
                                        }
                                    }
                                    OutputsToCollect::All => {
                                        output_ids_to_collect.insert(output_data.output_id);
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
                                    &account.addresses_with_balance,
                                    output,
                                    local_time as u32,
                                    milestone_index,
                                )
                            {
                                match outputs_to_collect {
                                    OutputsToCollect::MicroTransactions => {
                                        if let Some(UnlockCondition::StorageDepositReturn(sdr)) = nft_output
                                            .unlock_conditions()
                                            .get(StorageDepositReturnUnlockCondition::KIND)
                                        {
                                            // Only micro transaction if not the same
                                            if sdr.amount() != nft_output.amount() {
                                                output_ids_to_collect.insert(output_data.output_id);
                                            }
                                        }
                                    }
                                    OutputsToCollect::NativeTokens => {
                                        if !nft_output.native_tokens().is_empty() {
                                            output_ids_to_collect.insert(output_data.output_id);
                                        }
                                    }
                                    OutputsToCollect::Nfts | OutputsToCollect::All => {
                                        output_ids_to_collect.insert(output_data.output_id);
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
            "[OUTPUT_COLLECTION] available outputs to collect: {}",
            output_ids_to_collect.len()
        );
        Ok(output_ids_to_collect.into_iter().collect())
    }

    /// Try to collect basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub async fn try_collect_outputs(
        &self,
        outputs_to_collect: OutputsToCollect,
    ) -> crate::Result<Vec<TransferResult>> {
        log::debug!("[OUTPUT_COLLECTION] try_collect_outputs");

        let output_ids_to_collect = self
            .get_outputs_with_additional_unlock_conditions(outputs_to_collect)
            .await?;
        let basic_outputs = self.get_basic_outputs_for_additional_inputs().await?;
        self.collect_outputs_internal(output_ids_to_collect, basic_outputs)
            .await
    }

    /// Get basic outputs that have only one unlock condition which is [AddressUnlockCondition], so they can be used as
    /// additional inputs
    pub async fn get_basic_outputs_for_additional_inputs(&self) -> crate::Result<Vec<OutputData>> {
        log::debug!("[OUTPUT_COLLECTION] get_basic_outputs_for_additional_inputs");
        let account = self.read().await;

        let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;

        // Get basic outputs only with AddressUnlockCondition and no other unlock condition
        let mut basic_outputs: Vec<OutputData> = Vec::new();
        for (output_id, output_data) in &account.unspent_outputs {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                if let Some(output) = account.outputs.get(output_id) {
                    if let Output::Basic(basic_output) = &output.output {
                        if basic_output.unlock_conditions().len() == 1 {
                            // Store outputs with [AddressUnlockCondition] alone, because they could be used as
                            // additional input, if required
                            basic_outputs.push(output_data.clone());
                        }
                    }
                }
            }
        }
        log::debug!("[OUTPUT_COLLECTION] available basic outputs: {}", basic_outputs.len());
        Ok(basic_outputs)
    }

    /// Try to collect basic or nft outputs that have additional unlock conditions to their [AddressUnlockCondition]
    /// from [`get_outputs_with_additional_unlock_conditions()`].
    pub async fn collect_outputs(&self, output_ids_to_collect: Vec<OutputId>) -> crate::Result<Vec<TransferResult>> {
        log::debug!("[OUTPUT_COLLECTION] collect_outputs");
        let basic_outputs = self.get_basic_outputs_for_additional_inputs().await?;
        self.collect_outputs_internal(output_ids_to_collect, basic_outputs)
            .await
    }

    /// Try to collect basic outputs that have additional unlock conditions to their [AddressUnlockCondition].
    pub(crate) async fn collect_outputs_internal(
        &self,
        output_ids_to_collect: Vec<OutputId>,
        possible_additional_inputs: Vec<OutputData>,
    ) -> crate::Result<Vec<TransferResult>> {
        log::debug!("[OUTPUT_COLLECTION] collect_outputs_internal");
        let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let mut outputs_to_collect = Vec::new();
        let account = self.read().await;
        for output_id in output_ids_to_collect {
            if let Some(output_data) = account.unspent_outputs.get(&output_id) {
                outputs_to_collect.push(output_data.clone());
            }
        }

        if outputs_to_collect.is_empty() {
            // No outputs to collect, return
            return Ok(Vec::new());
        }

        let first_account_address = account
            .public_addresses
            .first()
            .ok_or(crate::Error::FailedToGetRemainder)?
            .clone();
        drop(account);

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
            // Amount we get with the storage deposit return amounts already subtracted
            let mut new_amount = 0;
            let mut new_native_tokens = HashMap::new();
            // check native tokens
            for output_data in outputs {
                if let Output::Nft(nft_output) = &output_data.output {
                    // build new output with same amount, nft_id, immutable/feature blocks and native tokens, just
                    // updated address unlock conditions

                    // todo: use minimum storage deposit amount for amount
                    let mut nft_builder = NftOutputBuilder::new(
                        nft_output.amount(),
                        nft_output.nft_id().or_from_output_id(output_data.output_id),
                    )?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        first_account_address.address.inner,
                    )));
                    // native tokens are added later
                    for feature_block in nft_output.feature_blocks().iter() {
                        nft_builder = nft_builder.add_feature_block(feature_block.clone());
                    }
                    for immutable_feature_block in nft_output.immutable_feature_blocks().iter() {
                        nft_builder = nft_builder.add_immutable_feature_block(immutable_feature_block.clone());
                    }
                    outputs_to_send.push(Output::Nft(nft_builder.finish()?));
                }

                // if expired, we can send everything to us
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

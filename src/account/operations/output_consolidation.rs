// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    input::INPUT_COUNT_MAX,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder, NativeTokens, NativeTokensBuilder, Output,
    },
};
#[cfg(feature = "ledger_nano")]
use iota_client::secret::SecretManager;

// Constants for the calculation of the amount of inputs we can use with a ledger nano
#[cfg(feature = "ledger_nano")]
const ESSENCE_SIZE_WITHOUT_IN_AND_OUTPUTS: usize = 49;
#[cfg(feature = "ledger_nano")]
// Input size in essence (35) + LedgerBIP32Index (8)
const INPUT_SIZE: usize = 43;
#[cfg(feature = "ledger_nano")]
const MIN_OUTPUT_SIZE_IN_ESSENCE: usize = 46;

#[cfg(feature = "ledger_nano")]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
use crate::account::{
    constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle,
    operations::output_claiming::get_new_native_token_count, types::Transaction, TransactionOptions,
};

impl AccountHandle {
    /// Consolidate basic outputs with only an [AddressUnlockCondition] from an account by sending them to an own
    /// address again if the output amount is >= the output_consolidation_threshold. When `force` is set to `true`, the
    /// threshold is ignored. Only consolidates the amount of outputs that fit into a single transaction.
    pub async fn consolidate_outputs(
        self: &AccountHandle,
        force: bool,
        output_consolidation_threshold: Option<usize>,
    ) -> crate::Result<Transaction> {
        log::debug!("[OUTPUT_CONSOLIDATION] consolidating outputs if needed");
        let account = self.read().await;
        let token_supply = self.client.get_token_supply().await?;

        // Get outputs for the consolidation
        let mut outputs_to_consolidate = Vec::new();
        for (output_id, output_data) in account.unspent_outputs() {
            // Don't use outputs that are locked for other transactions
            if !account.locked_outputs.contains(output_id) {
                // Only consolidate basic outputs with the address unlock condition alone
                if let Output::Basic(basic_output) = &output_data.output {
                    if let [UnlockCondition::Address(_)] = &basic_output.unlock_conditions().as_ref() {
                        outputs_to_consolidate.push(output_data.clone());
                    }
                }
            }
        }

        drop(account);

        let output_consolidation_threshold = output_consolidation_threshold.unwrap_or({
            match &*self.secret_manager.read().await {
                #[cfg(feature = "ledger_nano")]
                SecretManager::LedgerNano(_) => DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD,
                _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
            }
        });

        // only consolidate if the unlocked outputs are >= output_consolidation_threshold
        if outputs_to_consolidate.is_empty()
            || (!force && outputs_to_consolidate.len() < output_consolidation_threshold)
        {
            log::debug!(
                "[OUTPUT_CONSOLIDATION] no consolidation needed, available_outputs: {}, consolidation_threshold: {}",
                outputs_to_consolidate.len(),
                output_consolidation_threshold
            );
            return Err(crate::Error::NoOutputsToConsolidate {
                available_outputs: outputs_to_consolidate.len(),
                consolidation_threshold: output_consolidation_threshold,
            });
        }

        let max_inputs = match &*self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger) => {
                let ledger_nano_status = ledger.get_ledger_nano_status().await;
                // With blind signing we are only limited by the protocol
                if ledger_nano_status.blind_signing_enabled() {
                    INPUT_COUNT_MAX
                } else {
                    ledger_nano_status
                        .buffer_size()
                        .map(|buffer_size| {
                            // Calculate how many inputs we can have with this ledger, buffer size is different for
                            // different ledger types
                            let available_buffer_size_for_inputs =
                                buffer_size - ESSENCE_SIZE_WITHOUT_IN_AND_OUTPUTS - MIN_OUTPUT_SIZE_IN_ESSENCE;
                            (available_buffer_size_for_inputs / INPUT_SIZE) as u16
                        })
                        .unwrap_or(INPUT_COUNT_MAX)
                }
            }
            _ => INPUT_COUNT_MAX,
        };

        let mut total_amount = 0;
        let mut custom_inputs = Vec::with_capacity(max_inputs.into());
        let mut total_native_tokens = NativeTokensBuilder::new();

        for output_data in outputs_to_consolidate.iter().take(max_inputs.into()) {
            if let Some(native_tokens) = output_data.output.native_tokens() {
                // Skip output if the max native tokens count would be exceeded
                if get_new_native_token_count(&total_native_tokens, native_tokens)? > NativeTokens::COUNT_MAX.into() {
                    log::debug!("[OUTPUT_CONSOLIDATION] skipping output to not exceed the max native tokens count");
                    continue;
                }
                total_native_tokens.add_native_tokens(native_tokens.clone())?;
            };
            total_amount += output_data.output.amount();

            custom_inputs.push(output_data.output_id);
        }

        let consolidation_output = vec![
            BasicOutputBuilder::new_with_amount(total_amount)?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    outputs_to_consolidate[0].address,
                )))
                .with_native_tokens(total_native_tokens.finish()?)
                .finish_output(token_supply)?,
        ];

        let consolidation_tx = self
            .finish_transaction(
                consolidation_output,
                Some(TransactionOptions {
                    custom_inputs: Some(custom_inputs),
                    ..Default::default()
                }),
            )
            .await?;

        log::debug!(
            "[OUTPUT_CONSOLIDATION] consolidation transaction created: block_id: {:?} tx_id: {:?}",
            consolidation_tx.block_id,
            consolidation_tx.transaction_id
        );

        Ok(consolidation_tx)
    }
}

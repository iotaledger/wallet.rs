// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_block::output::{
    unlock_condition::{AddressUnlockCondition, UnlockCondition},
    BasicOutputBuilder, NativeTokensBuilder, Output,
};
#[cfg(feature = "ledger_nano")]
use iota_client::secret::SecretManager;

#[cfg(feature = "ledger_nano")]
use crate::account::constants::DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD;
use crate::account::{
    constants::DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD, handle::AccountHandle,
    operations::transaction::TransactionResult, types::address::AddressWithUnspentOutputs, TransactionOptions,
};

impl AccountHandle {
    /// Consolidates basic outputs with only an [AddressUnlockCondition] from an account by sending them to the same
    /// address again if the output amount is >= the output_consolidation_threshold
    pub async fn consolidate_outputs(
        self: &AccountHandle,
        force: bool,
        output_consolidation_threshold: Option<usize>,
    ) -> crate::Result<Vec<TransactionResult>> {
        let account = self.read().await;
        let output_consolidation_threshold = match output_consolidation_threshold {
            Some(threshold) => threshold,
            None => match *self.secret_manager.read().await {
                #[cfg(feature = "ledger_nano")]
                SecretManager::LedgerNano(_) | SecretManager::LedgerNanoSimulator(_) => {
                    DEFAULT_LEDGER_OUTPUT_CONSOLIDATION_THRESHOLD
                }
                _ => DEFAULT_OUTPUT_CONSOLIDATION_THRESHOLD,
            },
        };
        let addresses_that_need_consolidation: Vec<&AddressWithUnspentOutputs> = account
            .addresses_with_unspent_outputs
            .iter()
            .filter(|a| force || a.output_ids.len() >= output_consolidation_threshold)
            .collect();

        if addresses_that_need_consolidation.is_empty() {
            log::debug!("[OUTPUT_CONSOLIDATION] no consolidation needed");
            return Ok(Vec::new());
        }
        log::debug!("[OUTPUT_CONSOLIDATION] consolidating outputs if needed");

        // Get outputs for the consolidation
        let mut outputs_to_consolidate = Vec::new();
        for address in addresses_that_need_consolidation {
            let mut unspent_outputs = Vec::new();
            for output_id in &address.output_ids {
                // Don't use outputs that are locked for other transactions
                if !account.locked_outputs.contains(output_id) {
                    if let Some(output) = account.outputs.get(output_id) {
                        // Only consolidate basic outputs with no address unlock condition alone
                        if let Output::Basic(basic_output) = &output.output {
                            // If the length is 1, then there is only [AddressUnlockCondition]
                            if basic_output.unlock_conditions().len() == 1 {
                                unspent_outputs.push(output.clone());
                            }
                        }
                    }
                }
            }
            // only consolidate if the unlocked outputs are >= output_consolidation_threshold
            if force || unspent_outputs.len() >= output_consolidation_threshold {
                log::debug!(
                    "[OUTPUT_CONSOLIDATION] {} has {} unspent basic outputs with only an AddressUnlockCondition",
                    address.address.to_bech32(),
                    unspent_outputs.len()
                );
                outputs_to_consolidate.push(unspent_outputs);
            }
        }
        drop(account);

        if outputs_to_consolidate.is_empty() {
            log::debug!("[OUTPUT_CONSOLIDATION] no consolidation needed");
        }

        let mut consolidation_results = Vec::new();
        for outputs_on_one_address in outputs_to_consolidate {
            // todo: remove magic number and get a value that works for the current secret_manager (ledger is limited)
            // and is <= max inputs
            for outputs in outputs_on_one_address.chunks(16) {
                let mut total_amount = 0;
                let mut custom_inputs = Vec::with_capacity(16);
                let mut total_native_tokens = NativeTokensBuilder::new();
                for output_data in outputs {
                    total_amount += output_data.amount;

                    custom_inputs.push(output_data.output_id);

                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        total_native_tokens.add_native_tokens(native_tokens.clone())?;
                    };
                }

                let consolidation_output = vec![
                    BasicOutputBuilder::new_with_amount(total_amount)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                            outputs[0].address,
                        )))
                        .with_native_tokens(total_native_tokens.finish()?)
                        .finish_output()?,
                ];

                match self
                    .finish_transaction(
                        consolidation_output,
                        Some(TransactionOptions {
                            skip_sync: true,
                            custom_inputs: Some(custom_inputs),
                            ..Default::default()
                        }),
                    )
                    .await
                {
                    Ok(res) => {
                        log::debug!(
                            "[OUTPUT_CONSOLIDATION] Consolidation transaction created: block_id: {:?} tx_id: {:?}",
                            res.transaction.block_id,
                            res.transaction_id
                        );
                        consolidation_results.push(res);
                    }
                    Err(e) => log::debug!("Consolidation error: {}", e),
                };
            }
        }

        Ok(consolidation_results)
    }
}

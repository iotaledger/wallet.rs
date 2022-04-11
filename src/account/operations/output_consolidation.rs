// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    handle::AccountHandle,
    operations::transfer::TransferResult,
    types::{address::AddressWithBalance, OutputData},
    TransferOptions,
};

use iota_client::bee_message::output::{
    unlock_condition::{AddressUnlockCondition, UnlockCondition},
    BasicOutputBuilder, Output, OutputId,
};

impl AccountHandle {
    /// Consolidates basic outputs with only an [AddressUnlockCondition] from an account by sending them to the same
    /// address again if the output amount is >= the output_consolidation_threshold
    pub(crate) async fn consolidate_outputs(self: &AccountHandle) -> crate::Result<Vec<TransferResult>> {
        let account = self.read().await;
        let output_consolidation_threshold = account.account_options.output_consolidation_threshold;
        let addresses_that_need_consolidation: Vec<&AddressWithBalance> = account
            .addresses_with_balance
            .iter()
            .filter(|a| a.output_ids.len() >= output_consolidation_threshold)
            .collect();

        if addresses_that_need_consolidation.is_empty() {
            log::debug!("[OUTPUT_CONSOLIDATION] no consolidation needed");
            return Ok(Vec::new());
        }
        log::debug!("[OUTPUT_CONSOLIDATION] consolidating outputs if needed");

        // Get outputs for the consolidation
        let mut outputs_to_consolidate: Vec<Vec<OutputData>> = Vec::new();
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
            if unspent_outputs.len() >= output_consolidation_threshold {
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

        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let mut consolidation_results = Vec::new();
        for outputs_on_one_address in outputs_to_consolidate {
            // todo: remove magic number and get a value that works for the current signer (ledger is limited) and is <=
            // max inputs
            for outputs in outputs_on_one_address.chunks(16) {
                let output_sum = outputs.iter().map(|o| o.amount).sum();
                let consolidation_output = vec![Output::Basic(
                    BasicOutputBuilder::new_with_amount(output_sum)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                            outputs[0].address,
                        )))
                        .finish()?,
                )];
                match self
                    .send_transfer(
                        consolidation_output,
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
                            "[OUTPUT_CONSOLIDATION] Consolidation transaction created: msg_id: {:?} tx_id: {:?}",
                            res.message_id,
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

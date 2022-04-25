// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, types::OutputData, TransactionOptions},
    Error,
};

use iota_client::bee_block::output::{AliasId, AliasOutputBuilder, FoundryId, Output};

impl AccountHandle {
    /// Function to destroy foundry
    pub async fn burn_foundry(
        &self,
        foundry_id: FoundryId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] destroy_foundry");

        let alias_id = *foundry_id.alias_address().alias_id();
        let (existing_alias_output_data, existing_foundry_output_data) =
            self.find_alias_and_foundry_output_data(alias_id, foundry_id).await?;

        let custom_inputs = vec![
            existing_alias_output_data.output_id,
            existing_foundry_output_data.output_id,
        ];

        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };

        let outputs = match existing_alias_output_data.output {
            Output::Alias(alias_output) => {
                // Amount in foundry can't be burned, only native tokens
                let amount = existing_alias_output_data.amount + existing_foundry_output_data.amount;
                // Create the new alias output with the same feature blocks, just updated amount and state_index
                let alias_output = AliasOutputBuilder::new_with_amount(amount, alias_id)?
                    .with_native_tokens(alias_output.native_tokens().clone())
                    .with_state_index(alias_output.state_index() + 1)
                    .with_state_metadata(alias_output.state_metadata().to_vec())
                    .with_foundry_counter(alias_output.foundry_counter())
                    .with_unlock_conditions(alias_output.unlock_conditions().clone())
                    .with_feature_blocks(alias_output.feature_blocks().clone())
                    .with_immutable_feature_blocks(alias_output.immutable_feature_blocks().clone())
                    .finish()?;

                vec![Output::Alias(alias_output)]
            }
            _ => unreachable!("We checked if it's an alias output before"),
        };

        self.send(outputs, options).await
    }

    /// Function to destroy foundries
    pub async fn burn_foundries(
        &self,
        foundry_ids: HashSet<FoundryId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] destroy_foundries");

        let mut existing_alias_and_foundry_output_data = Vec::new();

        for foundry_id in foundry_ids {
            let alias_id = *foundry_id.alias_address().alias_id();
            existing_alias_and_foundry_output_data.push(self.find_alias_and_foundry_output_data(alias_id, foundry_id).await?);
        }

        let mut custom_inputs = Vec::new();

        for (existing_alias_output_data, existing_foundry_output_data) in existing_alias_and_foundry_output_data.iter() {
            custom_inputs.push(existing_alias_output_data.output_id);
            custom_inputs.push(existing_foundry_output_data.output_id);
        }

        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };

        let mut outputs = Vec::new();

        for (existing_alias_output_data, existing_foundry_output_data) in existing_alias_and_foundry_output_data {
            match existing_alias_output_data.output {
                Output::Alias(alias_output) => {
                    // Amount in foundry can't be burned, only native tokens
                    let amount = existing_alias_output_data.amount + existing_foundry_output_data.amount;
                    // Create the new alias output with the same feature blocks, just updated amount and state_index
                    let alias_output = AliasOutputBuilder::new_with_amount(amount, *alias_output.alias_id())?
                        .with_native_tokens(alias_output.native_tokens().clone())
                        .with_state_index(alias_output.state_index() + 1)
                        .with_state_metadata(alias_output.state_metadata().to_vec())
                        .with_foundry_counter(alias_output.foundry_counter())
                        .with_unlock_conditions(alias_output.unlock_conditions().clone())
                        .with_feature_blocks(alias_output.feature_blocks().clone())
                        .with_immutable_feature_blocks(alias_output.immutable_feature_blocks().clone())
                        .finish()?;

                    outputs.push(Output::Alias(alias_output));
                }
                _ => unreachable!("We checked if it's an alias output before"),
            };
        }
        

        self.send(outputs, options).await
    }

    pub(crate) async fn find_alias_and_foundry_output_data(
        &self,
        alias_id: AliasId,
        foundry_id: FoundryId,
    ) -> crate::Result<(OutputData, OutputData)> {
        let account = self.read().await;

        let mut existing_alias_output_data = None;
        let mut existing_foundry_output = None;
        account
            .unspent_outputs()
            .values()
            .into_iter()
            .filter(|output_data| match &output_data.output {
                Output::Alias(output) => output.alias_id().or_from_output_id(output_data.output_id) == alias_id,
                Output::Foundry(output) => output.id() == foundry_id,
                _ => false,
            })
            .for_each(|output_data| match &output_data.output {
                Output::Alias(_) => existing_alias_output_data = Some(output_data),
                Output::Foundry(_) => existing_foundry_output = Some(output_data),
                _ => unreachable!("We checked if it's an alias or foundry output before"),
            });

        let existing_alias_output_data = existing_alias_output_data
            .ok_or_else(|| Error::BurningFailed("Required alias output for foundry not found".to_string()))?
            .clone();

        let existing_foundry_output_data = existing_foundry_output
            .ok_or_else(|| Error::BurningFailed("Required foundry output not found".to_string()))?
            .clone();

        Ok((existing_alias_output_data, existing_foundry_output_data))
    }
}

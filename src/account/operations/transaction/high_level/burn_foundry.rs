// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::{
    account::{
        handle::AccountHandle, operations::transfer::TransferResult, types::OutputData, SyncOptions, TransferOptions,
    },
    Error,
};

use iota_client::bee_message::{
    output::{AliasId, AliasOutputBuilder, FoundryId, Output, OUTPUT_COUNT_MAX},
    payload::transaction::TransactionId,
};

impl AccountHandle {
    /// Function to destroy foundry
    pub async fn burn_foundry(
        &self,
        foundry_id: FoundryId,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] burn_foundry");

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
                let mut native_tokens = Vec::from_iter(alias_output.native_tokens().clone());
                let burn_native_token_remainder = options.as_ref().map_or(false, |options| options.allow_burning);
                if !burn_native_token_remainder {
                    // Transfer native tokens from foundry to alias
                    if let Output::Foundry(foundry_output) = existing_foundry_output_data.output {
                        native_tokens.extend(foundry_output.native_tokens().clone())
                    } else {
                        unreachable!("We already checked output is a foundry");
                    }
                };
                // Create the new alias output with updated amount, state_index and native token if not burning foundry tokens
                let alias_output = AliasOutputBuilder::from(&alias_output)
                    .with_alias_id(alias_id)
                    .with_amount(amount)?
                    .with_native_tokens(native_tokens)
                    .with_state_index(alias_output.state_index() + 1)
                    .finish()?;

                vec![Output::Alias(alias_output)]
            }
            _ => unreachable!("We checked if it's an alias output before"),
        };

        self.send(outputs, options).await
    }

    /// Burn all the foundries in the given set `foundry_ids`
    pub async fn burn_foundries(
        &self,
        foundry_ids: HashSet<FoundryId>,
<<<<<<< HEAD:src/account/operations/transaction/high_level/burn_foundry.rs
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] destroy_foundries");
=======
        options: Option<TransferOptions>,
    ) -> crate::Result<Vec<TransactionId>> {
        log::debug!("[TRANSFER] burn_foundries");
>>>>>>> 74560f74 (Fix destroy alias and refactor):src/account/operations/transfer/high_level/burn_foundry.rs

        let foundries = foundry_ids.into_iter().collect::<Vec<_>>();
        let mut transaction_ids = Vec::new();

        for foundry_ids in foundries.chunks(OUTPUT_COUNT_MAX as usize) {
            let mut existing_alias_and_foundry_output_data = Vec::new();

            for foundry_id in foundry_ids {
                let alias_id = *foundry_id.alias_address().alias_id();
                let alias_and_foundry_output_data =
                    self.find_alias_and_foundry_output_data(alias_id, *foundry_id).await?;
                existing_alias_and_foundry_output_data.push(alias_and_foundry_output_data);
            }
<<<<<<< HEAD:src/account/operations/transaction/high_level/burn_foundry.rs
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };
=======
>>>>>>> 1ac57c05 (Recursively transfer alias and nft address outputs):src/account/operations/transfer/high_level/burn_foundry.rs

            let mut custom_inputs = Vec::new();

            for (alias_output_data, foundry_output_data) in existing_alias_and_foundry_output_data.iter() {
                custom_inputs.push(alias_output_data.output_id);
                custom_inputs.push(foundry_output_data.output_id);
            }

            let options = match options.clone() {
                Some(mut options) => {
                    options.custom_inputs.replace(custom_inputs);
                    Some(options)
                }
                None => Some(TransferOptions {
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
                        let mut native_tokens = Vec::from_iter(alias_output.native_tokens().clone());
                        let burn_native_token_remainder =
                            options.as_ref().map_or(false, |options| options.allow_burning);
                        if !burn_native_token_remainder {
                            // Transfer native tokens from foundry to alias
                            if let Output::Foundry(foundry_output) = existing_foundry_output_data.output {
                                native_tokens.extend(foundry_output.native_tokens().clone())
                            } else {
                                unreachable!("We already checked output is a foundry");
                            }
                        };

                        // Create the new alias output with updated amount, state_index and native token if not burning foundry tokens
                        let alias_output = AliasOutputBuilder::from(&alias_output)
                            .with_amount(amount)?
                            .with_native_tokens(native_tokens)
                            .with_state_index(alias_output.state_index() + 1)
                            .finish()?;

                        outputs.push(Output::Alias(alias_output));
                    }
                    _ => unreachable!("We checked if it's an alias output before"),
                };
            }

            let transfer_result = self.send(outputs, options).await?;
            transaction_ids.push(transfer_result.transaction_id);
            match transfer_result.message_id {
                Some(message_id) => {
                    let _ = self.client.retry_until_included(&message_id, None, None).await?;
                    let sync_options = Some(SyncOptions {
                        force_syncing: true,
                        ..Default::default()
                    });
                    let _ = self.sync(sync_options).await?;
                }
                None => return Err(Error::BurningFailed("Could not burn foundries".to_string())),
            }
        }

        Ok(transaction_ids)
    }

    /// Find and return unspent `OutputData` for given `alias_id` and `foundry_id`
    pub(super) async fn find_alias_and_foundry_output_data(
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

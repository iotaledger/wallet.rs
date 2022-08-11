// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use iota_client::block::{
    output::{AliasId, AliasOutputBuilder, FoundryId, NativeTokensBuilder, Output, TokenScheme, OUTPUT_COUNT_MAX},
    payload::transaction::TransactionId,
};

use crate::{
    account::{
        handle::AccountHandle, operations::transaction::Transaction, types::OutputData, SyncOptions, TransactionOptions,
    },
    Error,
};

impl AccountHandle {
    /// Function to destroy a foundry output with a circulating supply of 0.
    /// Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias
    pub async fn destroy_foundry(
        &self,
        foundry_id: FoundryId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] destroy_foundry");

        let alias_id = *foundry_id.alias_address().alias_id();
        let (existing_alias_output_data, existing_foundry_output_data) =
            self.find_alias_and_foundry_output_data(alias_id, foundry_id).await?;

        validate_empty_state(&existing_foundry_output_data.output)?;

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
                let amount = alias_output.amount() + existing_foundry_output_data.output.amount();
                let mut native_tokens_builder = NativeTokensBuilder::from(alias_output.native_tokens().clone());
                // Transfer native tokens from foundry to alias
                if let Output::Foundry(foundry_output) = existing_foundry_output_data.output {
                    native_tokens_builder.add_native_tokens(foundry_output.native_tokens().clone())?;
                } else {
                    unreachable!("We already checked output is a foundry");
                }
                // Create the new alias output with updated amount, state_index and native token if not burning foundry
                // tokens
                let alias_output = AliasOutputBuilder::from(&alias_output)
                    .with_alias_id(alias_id)
                    .with_amount(amount)?
                    .with_native_tokens(native_tokens_builder.finish()?)
                    .with_state_index(alias_output.state_index() + 1)
                    .finish()?;

                vec![Output::Alias(alias_output)]
            }
            _ => unreachable!("We checked if it's an alias output before"),
        };

        self.send(outputs, options).await
    }

    /// Destroy all foundries in the given set `foundry_ids`
    // TODO: allow destroying of multiple foundries of the same alias, currently only one foundry output per alias can
    // be destroyed when calling it once
    pub async fn destroy_foundries(
        &self,
        foundry_ids: HashSet<FoundryId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Vec<TransactionId>> {
        log::debug!("[TRANSACTION] destroy_foundries");

        let mut transaction_ids = Vec::new();
        let foundries = foundry_ids.into_iter().collect::<Vec<_>>();

        for foundry_ids in foundries.chunks(OUTPUT_COUNT_MAX as usize) {
            let mut custom_inputs = Vec::new();
            let mut outputs = Vec::new();
            let mut included_aliases = HashSet::new();

            for foundry_id in foundry_ids {
                let alias_id = *foundry_id.alias_address().alias_id();
                let (alias_output_data, foundry_output_data) =
                    self.find_alias_and_foundry_output_data(alias_id, *foundry_id).await?;

                validate_empty_state(&foundry_output_data.output)?;

                // To burn foundries we need the controlling alias to go into the inputs as well
                if !included_aliases.contains(&alias_id) {
                    included_aliases.insert(alias_id);
                    custom_inputs.push(alias_output_data.output_id);

                    // Alias output state transition
                    if let Output::Alias(alias_output) = alias_output_data.output {
                        let amount = alias_output.amount() + foundry_output_data.output.amount();
                        let mut native_tokens_builder = NativeTokensBuilder::from(alias_output.native_tokens().clone());
                        // Transfer native tokens from foundry to alias
                        if let Output::Foundry(foundry_output) = foundry_output_data.output {
                            native_tokens_builder.add_native_tokens(foundry_output.native_tokens().clone())?;
                        } else {
                            unreachable!("We already checked output is a foundry");
                        }

                        // Create the new alias output with updated amount, state_index and native token
                        let alias_output = AliasOutputBuilder::from(&alias_output)
                            .with_alias_id(alias_output.alias_id().or_from_output_id(alias_output_data.output_id))
                            .with_amount(amount)?
                            .with_native_tokens(native_tokens_builder.finish()?)
                            .with_state_index(alias_output.state_index() + 1)
                            .finish()?;

                        outputs.push(Output::Alias(alias_output));
                    } else {
                        unreachable!("We checked if it's an alias output before");
                    }
                }
                custom_inputs.push(foundry_output_data.output_id);
            }

            let options = match options.clone() {
                Some(mut options) => {
                    options.custom_inputs.replace(custom_inputs);
                    Some(options)
                }
                None => Some(TransactionOptions {
                    custom_inputs: Some(custom_inputs),
                    ..Default::default()
                }),
            };

            let transaction = self.send(outputs, options).await?;
            transaction_ids.push(transaction.transaction_id);
            match transaction.block_id {
                Some(block_id) => {
                    let _ = self.client.retry_until_included(&block_id, None, None).await?;
                    let sync_options = Some(SyncOptions {
                        force_syncing: true,
                        ..Default::default()
                    });
                    let _ = self.sync(sync_options).await?;
                }
                None => return Err(Error::BurningOrMeltingFailed("could not burn foundries".to_string())),
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

        for (output_id, output_data) in account.unspent_outputs().iter() {
            match &output_data.output {
                Output::Alias(output) => {
                    if output.alias_id().or_from_output_id(*output_id) == alias_id {
                        existing_alias_output_data = Some(output_data);
                    }
                }
                Output::Foundry(output) => {
                    if output.id() == foundry_id {
                        existing_foundry_output = Some(output_data);
                    }
                }
                // Not interested in these outputs here
                Output::Treasury(_) | Output::Basic(_) | Output::Nft(_) => {}
            }

            if existing_alias_output_data.is_some() && existing_foundry_output.is_some() {
                break;
            }
        }

        let existing_alias_output_data = existing_alias_output_data
            .ok_or_else(|| Error::BurningOrMeltingFailed("required alias output for foundry not found".to_string()))?
            .clone();

        let existing_foundry_output_data = existing_foundry_output
            .ok_or_else(|| Error::BurningOrMeltingFailed("required foundry output not found".to_string()))?
            .clone();

        Ok((existing_alias_output_data, existing_foundry_output_data))
    }
}

// A foundry output can only be destroyed if the circulating_supply is zero. If native tokens got burned, it can never
// be destroyed.
fn validate_empty_state(output: &Output) -> crate::Result<()> {
    match output {
        Output::Foundry(foundry_output) => {
            let TokenScheme::Simple(token_scheme) = foundry_output.token_scheme();
            if token_scheme.circulating_supply().is_zero() {
                Ok(())
            } else {
                Err(Error::BurningOrMeltingFailed(
                    "foundry still has native tokens in circulation or native tokens were burned".to_string(),
                ))
            }
        }
        _ => Err(Error::BurningOrMeltingFailed(
            "invalid output type: expected foundry".to_string(),
        )),
    }
}

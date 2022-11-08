// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use iota_client::block::{
    input::INPUT_COUNT_MAX,
    output::{
        AliasId, AliasOutputBuilder, BasicOutputBuilder, FoundryOutputBuilder, NativeToken, NativeTokensBuilder,
        NftOutputBuilder, Output, OutputId, TokenId, OUTPUT_COUNT_MAX,
    },
};
use primitive_types::U256;

use crate::account::{
    handle::AccountHandle, operations::transaction::Transaction, types::OutputData, TransactionOptions,
};

const NATIVE_TOKEN_OVERFLOW: &str = "NativeTokensOverflow";

struct StrippedOutput {
    output_id: OutputId,
    amount: U256,
    output: Output,
}

impl StrippedOutput {
    fn new(output_id: OutputId, amount: U256, output: Output) -> Self {
        StrippedOutput {
            output_id,
            amount,
            output,
        }
    }
}

struct StrippedOutputAggregate {
    custom_inputs: Vec<OutputId>,
    amount: U256,
    outputs: Vec<Output>,
}

impl AccountHandle {
    /// Function to burn native tokens. This doesn't require the foundry output which minted them, but will not increase
    /// the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
    /// recommended to use `decrease_native_token_supply()`, if the foundry output is available.
    pub async fn burn_native_token(
        &self,
        token_id: TokenId,
        burn_amount: U256,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] burn_native_token");

        let StrippedOutputAggregate {
            custom_inputs,
            amount: _,
            outputs,
        } = self.get_burn_inputs_and_outputs(token_id, burn_amount).await?;

        // Set custom inputs and allow burning
        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                options.allow_burning = true;
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                allow_burning: true,
                ..Default::default()
            }),
        };

        self.send(outputs, options).await
    }

    // Get inputs with the required native token amount and create new outputs, just with the to be burned native token
    // amount removed.
    async fn get_burn_inputs_and_outputs(
        &self,
        token_id: TokenId,
        burn_token_amount: U256,
    ) -> crate::Result<StrippedOutputAggregate> {
        let account = self.read().await;
        let token_supply = self.client.get_token_supply().await?;

        let mut basic_and_nft_selection = Vec::new();

        // Will check foundries with aliases before adding them
        let mut alias_selection = HashMap::new();
        let mut foundry_selection = Vec::new();

        for (output_id, output_data) in account.unspent_outputs().iter() {
            match &output_data.output {
                Output::Basic(_) | Output::Nft(_) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data, token_supply)? {
                        basic_and_nft_selection.push(StrippedOutput::new(*output_id, amount, output));
                    }
                }
                Output::Alias(alias_output) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data, token_supply)? {
                        alias_selection.insert(
                            alias_output.alias_id_non_null(output_id),
                            StrippedOutput::new(*output_id, amount, output),
                        );
                    }
                }
                Output::Foundry(_) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data, token_supply)? {
                        foundry_selection.push(StrippedOutput::new(*output_id, amount, output));
                    }
                }
                Output::Treasury(_) => continue,
            }
        }

        drop(account);

        if basic_and_nft_selection.is_empty() && alias_selection.is_empty() && foundry_selection.is_empty() {
            return Err(crate::Error::BurningOrMeltingFailed(
                "native token not found".to_string(),
            ));
        }

        let aggregate = {
            // Select unspent outputs with native tokens that sum up to the required amount to be burned
            let mut basic_and_nft_aggregate =
                aggregate_stripped_outputs(token_id, burn_token_amount, basic_and_nft_selection, token_supply)?;

            if basic_and_nft_aggregate.amount < burn_token_amount {
                // Select more outputs from aliases and foundries if we don't have enough tokens to burn from basic and
                // nft outputs
                let alias_and_foundry_aggregate = self
                    .aggregate_stripped_alias_and_foundry_outputs(
                        token_id,
                        burn_token_amount - basic_and_nft_aggregate.amount,
                        foundry_selection,
                        alias_selection,
                    )
                    .await?;

                basic_and_nft_aggregate.amount = basic_and_nft_aggregate
                    .amount
                    .checked_add(alias_and_foundry_aggregate.amount)
                    .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
                basic_and_nft_aggregate
                    .custom_inputs
                    .extend(alias_and_foundry_aggregate.custom_inputs);
                basic_and_nft_aggregate
                    .outputs
                    .extend(alias_and_foundry_aggregate.outputs);
            }

            basic_and_nft_aggregate
        };

        if aggregate.amount < burn_token_amount {
            return Err(crate::Error::BurningOrMeltingFailed(format!(
                "insufficient native token balance: {}/{burn_token_amount}",
                aggregate.amount
            )));
        }
        if aggregate.outputs.len() > (OUTPUT_COUNT_MAX as usize) {
            return Err(crate::Error::BurningOrMeltingFailed(format!(
                "outputs for required amount exceed max allowed count: {}/{OUTPUT_COUNT_MAX}; try a lower amount",
                aggregate.outputs.len()
            )));
        }
        if aggregate.custom_inputs.len() > (INPUT_COUNT_MAX as usize) {
            return Err(crate::Error::BurningOrMeltingFailed(format!(
                "inputs for required amount exceed max allowed count: {}/{INPUT_COUNT_MAX}; try a lower amount",
                aggregate.custom_inputs.len()
            )));
        }

        Ok(aggregate)
    }

    /// Aggregate foundry and alias outputs that sum up to
    /// required `burn_token_amount`, if controlling alias doesn't have
    /// the specified native token it's added anyways for unlock purpose
    async fn aggregate_stripped_alias_and_foundry_outputs(
        &self,
        token_id: TokenId,
        burn_token_amount: U256,
        stripped_foundry_output: Vec<StrippedOutput>,
        stripped_alias_outputs: HashMap<AliasId, StrippedOutput>,
    ) -> crate::Result<StrippedOutputAggregate> {
        let token_supply = self.client.get_token_supply().await?;
        let mut stripped_foundry_outputs = stripped_foundry_output;
        let mut stripped_alias_outputs = stripped_alias_outputs;
        // Sort descending order based on token amount
        stripped_foundry_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));

        let mut aggregate = StrippedOutputAggregate {
            custom_inputs: Vec::new(),
            amount: U256::from(0i32),
            outputs: Vec::new(),
        };

        // Keep track of already included aliases because foundries can have the same controlling alias
        let mut included_alias = HashSet::new();

        for stripped_foundry_output in stripped_foundry_outputs.into_iter().take(OUTPUT_COUNT_MAX as usize) {
            // Add controlling alias
            if let Output::Foundry(foundry_output) = &stripped_foundry_output.output {
                let alias_id = *foundry_output.alias_address().alias_id();
                if !included_alias.contains(&alias_id) {
                    if !self
                        .add_controlling_alias_to_aggregate(alias_id, &mut aggregate, &mut stripped_alias_outputs)
                        .await?
                    {
                        // Can't find controlling alias for foundry
                        continue;
                    }
                    included_alias.insert(alias_id);
                }
            }

            // Add foundry
            aggregate.custom_inputs.push(stripped_foundry_output.output_id);
            aggregate.amount = aggregate
                .amount
                .checked_add(stripped_foundry_output.amount)
                .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;

            match aggregate.amount.cmp(&burn_token_amount) {
                Ordering::Less => aggregate.outputs.push(stripped_foundry_output.output),
                Ordering::Equal => {
                    aggregate.outputs.push(stripped_foundry_output.output);
                    break;
                }
                Ordering::Greater => {
                    // Add remaining native tokens back to an output, so we don't burn more than we should
                    let native_token = NativeToken::new(token_id, aggregate.amount - burn_token_amount)?;
                    let output =
                        add_native_token_to_output(&stripped_foundry_output.output, native_token, token_supply)?;
                    aggregate.outputs.push(output);
                    break;
                }
            }
        }

        if aggregate.amount < burn_token_amount {
            // Add remaining alias outputs
            let alias_aggregate = aggregate_stripped_outputs(
                token_id,
                aggregate.amount - burn_token_amount,
                stripped_alias_outputs.into_values().collect(),
                token_supply,
            )?;
            aggregate.amount = aggregate
                .amount
                .checked_add(alias_aggregate.amount)
                .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
            aggregate.custom_inputs.extend(alias_aggregate.custom_inputs);
            aggregate.outputs.extend(alias_aggregate.outputs);
        }

        Ok(aggregate)
    }

    async fn add_controlling_alias_to_aggregate(
        &self,
        alias_id: AliasId,
        aggregate: &mut StrippedOutputAggregate,
        stripped_alias_outputs: &mut HashMap<AliasId, StrippedOutput>,
    ) -> crate::Result<bool> {
        let token_supply = self.client.get_token_supply().await?;

        match stripped_alias_outputs.remove(&alias_id) {
            Some(StrippedOutput {
                output_id,
                amount,
                output,
            }) => {
                aggregate.amount = aggregate
                    .amount
                    .checked_add(amount)
                    .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
                aggregate.custom_inputs.push(output_id);
                aggregate.outputs.push(output);

                return Ok(true);
            }
            None => {
                // Find controlling alias
                let account = self.read().await;
                for (output_id, output_data) in account.unspent_outputs().iter() {
                    match &output_data.output {
                        Output::Alias(alias_output) => {
                            if alias_output.alias_id_non_null(output_id) == alias_id {
                                let alias_output = AliasOutputBuilder::from(alias_output)
                                    .with_alias_id(alias_output.alias_id_non_null(output_id))
                                    .with_state_index(alias_output.state_index() + 1)
                                    .finish_output(token_supply)?;
                                aggregate.custom_inputs.push(*output_id);
                                aggregate.outputs.push(alias_output);

                                return Ok(true);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }

        Ok(false)
    }
}

// If output has the to be burned native token, recreate the output with the native token removed and return the amount
// for the native token together with the new output.
fn strip_native_token_if_found(
    token_id: TokenId,
    output_data: &OutputData,
    token_supply: u64,
) -> crate::Result<Option<(U256, Output)>> {
    if let Some(native_tokens) = output_data.output.native_tokens() {
        let mut native_token_amount = U256::from(0);
        let mut not_to_be_stripped_native_tokens = NativeTokensBuilder::new();

        for native_token in native_tokens.iter() {
            if *native_token.token_id() == token_id {
                native_token_amount = native_token_amount
                    .checked_add(native_token.amount())
                    .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
            } else {
                not_to_be_stripped_native_tokens.add_native_token(native_token.clone())?;
            }
        }

        // If the output has native tokens that are to be deleted,
        if !native_token_amount.is_zero() {
            let output = create_output_and_replace_native_tokens(
                output_data,
                not_to_be_stripped_native_tokens.finish()?.into_iter(),
                token_supply,
            )?;
            return Ok(Some((native_token_amount, output)));
        }
    }

    Ok(None)
}

/// Aggregate outputs with `token_id` that sum up to the required burn_token_amount
fn aggregate_stripped_outputs(
    token_id: TokenId,
    burn_token_amount: U256,
    stripped_output: Vec<StrippedOutput>,
    token_supply: u64,
) -> crate::Result<StrippedOutputAggregate> {
    let mut stripped_outputs = stripped_output;
    // Sort descending order based on token amount
    stripped_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));

    let mut aggregate = StrippedOutputAggregate {
        custom_inputs: Vec::new(),
        amount: U256::from(0i32),
        outputs: Vec::new(),
    };

    for stripped_output in stripped_outputs.into_iter().take(OUTPUT_COUNT_MAX as usize) {
        aggregate.custom_inputs.push(stripped_output.output_id);
        aggregate.amount = aggregate
            .amount
            .checked_add(stripped_output.amount)
            .ok_or_else(|| crate::Error::BurningOrMeltingFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;

        match aggregate.amount.cmp(&burn_token_amount) {
            Ordering::Less => aggregate.outputs.push(stripped_output.output),
            Ordering::Equal => {
                aggregate.outputs.push(stripped_output.output);
                break;
            }
            Ordering::Greater => {
                // Add remaining native tokens back to an output, so we don't burn more than we should
                let native_token = NativeToken::new(token_id, aggregate.amount - burn_token_amount)?;
                let output = add_native_token_to_output(&stripped_output.output, native_token, token_supply)?;
                aggregate.outputs.push(output);
                break;
            }
        }
    }

    Ok(aggregate)
}

fn create_output_and_replace_native_tokens(
    output_data: &OutputData,
    native_tokens: impl IntoIterator<Item = NativeToken>,
    token_supply: u64,
) -> crate::Result<Output> {
    let output = match &output_data.output {
        Output::Alias(alias_output) => AliasOutputBuilder::from(alias_output)
            .with_alias_id(alias_output.alias_id_non_null(&output_data.output_id))
            .with_state_index(alias_output.state_index() + 1)
            .with_native_tokens(native_tokens)
            .finish_output(token_supply)?,
        Output::Basic(basic_output) => BasicOutputBuilder::from(basic_output)
            .with_native_tokens(native_tokens)
            .finish_output(token_supply)?,
        Output::Foundry(foundry_output) => FoundryOutputBuilder::from(foundry_output)
            .with_native_tokens(native_tokens)
            .finish_output(token_supply)?,
        Output::Nft(nft_output) => NftOutputBuilder::from(nft_output)
            .with_nft_id(nft_output.nft_id_non_null(&output_data.output_id))
            .with_native_tokens(native_tokens)
            .finish_output(token_supply)?,
        Output::Treasury(_) => {
            return Err(crate::Error::InvalidOutputKind(
                "treasury output cannot hold native tokens".to_string(),
            ));
        }
    };

    Ok(output)
}

fn add_native_token_to_output(output: &Output, native_token: NativeToken, token_supply: u64) -> crate::Result<Output> {
    let output = match &output {
        Output::Alias(alias_output) => AliasOutputBuilder::from(alias_output)
            .add_native_token(native_token)
            .finish_output(token_supply)?,
        Output::Basic(basic_output) => BasicOutputBuilder::from(basic_output)
            .add_native_token(native_token)
            .finish_output(token_supply)?,
        Output::Foundry(foundry_output) => FoundryOutputBuilder::from(foundry_output)
            .add_native_token(native_token)
            .finish_output(token_supply)?,
        Output::Nft(nft_output) => NftOutputBuilder::from(nft_output)
            .add_native_token(native_token)
            .finish_output(token_supply)?,
        Output::Treasury(_) => {
            return Err(crate::Error::InvalidOutputKind(
                "treasury output cannot hold native tokens".to_string(),
            ));
        }
    };

    Ok(output)
}

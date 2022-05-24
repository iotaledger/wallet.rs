// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use iota_client::bee_message::{
    address::AliasAddress,
    output::{
        unlock_condition::{ImmutableAliasAddressUnlockCondition, UnlockCondition},
        AliasId, AliasOutputBuilder, BasicOutputBuilder, FoundryOutputBuilder, NativeToken, NftOutputBuilder, Output,
        OutputId, SimpleTokenScheme, TokenId, TokenScheme, OUTPUT_COUNT_MAX,
    },
};
use primitive_types::U256;

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, types::OutputData, TransferOptions};

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
    /// Function to melt native tokens
    pub async fn melt_native_token(
        &self,
        native_token: (TokenId, U256),
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] melt_native_token");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let token_id = native_token.0;
        let burn_token_amount = native_token.1;

        let foundry_id = token_id.foundry_id();
        let alias_id = *foundry_id.alias_address().alias_id();

        let (existing_alias_output_data, existing_foundry_output) = self
            .find_alias_and_foundry_output_data(alias_id, foundry_id)
            .await
            .map(|(alias_data, foundry_data)| match foundry_data.output {
                Output::Foundry(foundry_output) => (alias_data, foundry_output),
                _ => unreachable!("We already checked it's a foundry output"),
            })?;

        if let Output::Alias(alias_output) = &existing_alias_output_data.output {
            // Amount can't be burned, only native tokens
            let amount = existing_alias_output_data.amount + existing_foundry_output.amount();
            // Create the new alias output with updated amount and state_index
            let alias_output = AliasOutputBuilder::from(alias_output)
                .with_alias_id(alias_id)
                .with_amount(amount)?
                .with_state_index(alias_output.state_index() + 1)
                .finish()?;

            let TokenScheme::Simple(foundry_simple_ts) = existing_foundry_output.token_scheme();
            let outputs = vec![
                Output::Alias(alias_output),
                Output::Foundry(
                    FoundryOutputBuilder::new_with_minimum_storage_deposit(
                        byte_cost_config,
                        foundry_id.serial_number(),
                        token_id.token_tag(),
                        TokenScheme::Simple(SimpleTokenScheme::new(
                            *foundry_simple_ts.minted_tokens(),
                            foundry_simple_ts.melted_tokens() + burn_token_amount,
                            *foundry_simple_ts.maximum_supply(),
                        )?),
                    )?
                    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
                        ImmutableAliasAddressUnlockCondition::new(AliasAddress::from(alias_id)),
                    ))
                    .finish()?,
                ),
            ];
            self.send(outputs, options).await
        } else {
            unreachable!("We checked if it's an alias output before")
        }
    }

    /// Function to burn native tokens
    pub async fn burn_native_token(
        &self,
        native_token: (TokenId, U256),
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] burn_native_token");
        let token_id = native_token.0;
        let burn_token_amount = native_token.1;

        let StrippedOutputAggregate {
            custom_inputs,
            amount: _,
            outputs,
        } = self.get_burn_inputs_and_outputs(token_id, burn_token_amount).await?;

        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                options.allow_burning = true;
                Some(options)
            }
            None => Some(TransferOptions {
                custom_inputs: Some(custom_inputs),
                allow_burning: true,
                ..Default::default()
            }),
        };

        self.send(outputs, options).await
    }

    async fn get_burn_inputs_and_outputs(
        &self,
        token_id: TokenId,
        burn_token_amount: U256,
    ) -> crate::Result<StrippedOutputAggregate> {
        let account = self.read().await;

        let mut basic_and_nft_selection = Vec::new();

        // Will check foundries with aliases before adding them
        let mut alias_selection = HashMap::new();
        let mut foundry_selection = Vec::new();

        for (output_id, output_data) in account.unspent_outputs().iter() {
            match &output_data.output {
                Output::Basic(_) | Output::Nft(_) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data)? {
                        basic_and_nft_selection.push(StrippedOutput::new(*output_id, amount, output));
                    }
                }
                Output::Alias(alias_output) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data)? {
                        alias_selection.insert(
                            alias_output.alias_id().or_from_output_id(*output_id),
                            StrippedOutput::new(*output_id, amount, output),
                        );
                    }
                }
                Output::Foundry(_) => {
                    if let Some((amount, output)) = strip_native_token_if_found(token_id, output_data)? {
                        foundry_selection.push(StrippedOutput::new(*output_id, amount, output));
                    }
                }
                Output::Treasury(_) => continue,
            }
        }

        drop(account);

        if basic_and_nft_selection.is_empty() && alias_selection.is_empty() && foundry_selection.is_empty() {
            return Err(crate::Error::BurningFailed("Native token not found".to_string()));
        }

        let aggregate = {
            // Select unspent outputs with native tokens that sum up to the required amount to be burned
            let mut basic_and_nft_aggregate =
                aggregate_stripped_outputs(token_id, burn_token_amount, basic_and_nft_selection)?;

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
                    .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
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
            return Err(crate::Error::BurningFailed(
                "Insufficient native token balance".to_string(),
            ));
        } else if aggregate.custom_inputs.len() > (OUTPUT_COUNT_MAX as usize) {
            return Err(crate::Error::BurningFailed(
                "Outputs for required amount exceed max allowed count; try a lower amount".to_string(),
            ));
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
                .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;

            match aggregate.amount.cmp(&burn_token_amount) {
                Ordering::Less => aggregate.outputs.push(stripped_foundry_output.output),
                Ordering::Equal => {
                    aggregate.outputs.push(stripped_foundry_output.output);
                    break;
                }
                Ordering::Greater => {
                    let native_token = NativeToken::new(token_id, aggregate.amount - burn_token_amount)?;
                    let output = add_native_token_to_output(&stripped_foundry_output.output, native_token)?;
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
            )?;
            aggregate.amount = aggregate
                .amount
                .checked_add(alias_aggregate.amount)
                .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
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
        match stripped_alias_outputs.remove(&alias_id) {
            Some(StrippedOutput {
                output_id,
                amount,
                output,
            }) => {
                aggregate.amount = aggregate
                    .amount
                    .checked_add(amount)
                    .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
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
                            if alias_output.alias_id().or_from_output_id(*output_id) == alias_id {
                                let alias_output = AliasOutputBuilder::from(alias_output)
                                    .with_alias_id(alias_output.alias_id().or_from_output_id(*output_id))
                                    .with_state_index(alias_output.state_index() + 1)
                                    .finish()?;
                                aggregate.custom_inputs.push(*output_id);
                                aggregate.outputs.push(Output::Alias(alias_output));

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

fn strip_native_token_if_found(token_id: TokenId, output_data: &OutputData) -> crate::Result<Option<(U256, Output)>> {
    if let Some(native_tokens) = output_data.output.native_tokens() {
        let mut amount = U256::from(0);
        let mut not_to_be_stripped_native_tokens = Vec::new();

        for native_token in native_tokens.iter() {
            if *native_token.token_id() == token_id {
                amount = amount
                    .checked_add(*native_token.amount())
                    .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;
            } else {
                not_to_be_stripped_native_tokens.push(native_token);
            }
        }

        // If the output has native tokens that are to be deleted,
        if !amount.is_zero() {
            let not_to_be_stripped_native_tokens = not_to_be_stripped_native_tokens.into_iter().cloned();
            let output = create_output_and_replace_native_tokens(output_data, not_to_be_stripped_native_tokens)?;
            return Ok(Some((amount, output)));
        }
    }

    Ok(None)
}
/// Aggregate outputs with `token_id` that sum up to the required amount
fn aggregate_stripped_outputs(
    token_id: TokenId,
    burn_token_amount: U256,
    stripped_output: Vec<StrippedOutput>,
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
            .ok_or_else(|| crate::Error::BurningFailed(NATIVE_TOKEN_OVERFLOW.to_string()))?;

        match aggregate.amount.cmp(&burn_token_amount) {
            Ordering::Less => aggregate.outputs.push(stripped_output.output),
            Ordering::Equal => {
                aggregate.outputs.push(stripped_output.output);
                break;
            }
            Ordering::Greater => {
                let native_token = NativeToken::new(token_id, aggregate.amount - burn_token_amount)?;
                let output = add_native_token_to_output(&stripped_output.output, native_token)?;
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
) -> crate::Result<Output> {
    let output = match &output_data.output {
        Output::Alias(alias_output) => {
            let alias_output = AliasOutputBuilder::from(alias_output)
                .with_alias_id(alias_output.alias_id().or_from_output_id(output_data.output_id))
                .with_state_index(alias_output.state_index() + 1)
                .with_native_tokens(native_tokens)
                .finish()?;
            Output::Alias(alias_output)
        }
        Output::Basic(basic_output) => {
            let output = BasicOutputBuilder::from(basic_output)
                .with_native_tokens(native_tokens)
                .finish()?;
            Output::Basic(output)
        }
        Output::Foundry(foundry_output) => {
            let output = FoundryOutputBuilder::from(foundry_output)
                .with_native_tokens(native_tokens)
                .finish()?;
            Output::Foundry(output)
        }
        Output::Nft(nft_output) => {
            let output = NftOutputBuilder::from(nft_output)
                .with_nft_id(nft_output.nft_id().or_from_output_id(output_data.output_id))
                .with_native_tokens(native_tokens)
                .finish()?;
            Output::Nft(output)
        }
        Output::Treasury(_) => {
            return Err(crate::Error::InvalidOutputKind(
                "Treasury output cannot hold native tokens".to_string(),
            ));
        }
    };

    Ok(output)
}

fn add_native_token_to_output(output: &Output, native_token: NativeToken) -> crate::Result<Output> {
    let output = match &output {
        Output::Alias(alias_output) => {
            let alias_output = AliasOutputBuilder::from(alias_output)
                .add_native_token(native_token)
                .finish()?;
            Output::Alias(alias_output)
        }
        Output::Basic(basic_output) => {
            let output = BasicOutputBuilder::from(basic_output)
                .add_native_token(native_token)
                .finish()?;
            Output::Basic(output)
        }
        Output::Foundry(foundry_output) => {
            let output = FoundryOutputBuilder::from(foundry_output)
                .add_native_token(native_token)
                .finish()?;
            Output::Foundry(output)
        }
        Output::Nft(nft_output) => {
            let output = NftOutputBuilder::from(nft_output)
                .add_native_token(native_token)
                .finish()?;
            Output::Nft(output)
        }
        Output::Treasury(_) => {
            return Err(crate::Error::InvalidOutputKind(
                "Treasury output cannot hold native tokens".to_string(),
            ));
        }
    };

    Ok(output)
}

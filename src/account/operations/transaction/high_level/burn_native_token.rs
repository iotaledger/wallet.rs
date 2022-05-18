// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use iota_client::bee_message::{
    address::AliasAddress,
    output::{
        unlock_condition::{ImmutableAliasAddressUnlockCondition, UnlockCondition},
        AliasOutputBuilder, BasicOutputBuilder, FoundryOutputBuilder, NativeToken, NftOutputBuilder, Output, OutputId,
        SimpleTokenScheme, TokenId, TokenScheme, OUTPUT_COUNT_MAX,
    },
};
use primitive_types::U256;

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

impl AccountHandle {
    /// Function to burn native tokens with foundry
    pub async fn burn_native_token(
        &self,
        native_token: (TokenId, U256),
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] burn_native_token");
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

    /// Function to burn native tokens without foundry
    pub async fn burn_native_token_without_foundry(
        &self,
        native_token: (TokenId, U256),
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] burn_native_token_without_foundry");
        let token_id = native_token.0;
        let burn_token_amount = native_token.1;

        let (custom_inputs, outputs) = self.select_native_token_output(token_id, burn_token_amount).await?;

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

    async fn select_native_token_output(
        &self,
        token_id: TokenId,
        burn_token_amount: U256,
    ) -> crate::Result<(Vec<OutputId>, Vec<Output>)> {
        let account = self.read().await;
        let mut inputs_and_outputs = Vec::new();
        for (output_id, output_data) in account.unspent_outputs().iter() {
            match output_data.output {
                Output::Alias(_) | Output::Basic(_) | Output::Nft(_) => {
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        let mut amount = U256::from(0);
                        let mut not_to_be_burnt_native_tokens = Vec::new();
                        for native_token in native_tokens.iter() {
                            if *native_token.token_id() == token_id {
                                amount += *native_token.amount();
                            } else {
                                not_to_be_burnt_native_tokens.push(native_token);
                            }
                        }

                        // If the output has a native token that we wish to burn,
                        // clone the output but without native tokens that are to be burnt
                        if !amount.is_zero() {
                            let not_to_be_burnt_native_tokens = not_to_be_burnt_native_tokens.into_iter().cloned();

                            let output =
                                replace_output_native_tokens(&output_data.output, not_to_be_burnt_native_tokens)?;
                            inputs_and_outputs.push((output_id, amount, output));
                        }
                    }
                }
                Output::Treasury(_) | Output::Foundry(_) => continue,
            }
        }

        if inputs_and_outputs.is_empty() {
            return Err(crate::Error::BurningFailed("Native token not found".to_string()));
        }

        // Sort descending order based on token amount
        inputs_and_outputs.sort_by(|a, b| b.1.cmp(&a.1));

        // Select unspent outputs with token id that sum up to the required amount
        let mut outputs = Vec::new();
        let mut custom_inputs = Vec::new();

        let mut native_token_amount_acc = U256::from(0);
        for input_and_output in inputs_and_outputs.into_iter().take(OUTPUT_COUNT_MAX as usize) {
            custom_inputs.push(*input_and_output.0);
            native_token_amount_acc += input_and_output.1;

            match native_token_amount_acc.cmp(&burn_token_amount) {
                Ordering::Less => outputs.push(input_and_output.2),
                Ordering::Equal => {
                    outputs.push(input_and_output.2);
                    break;
                }
                Ordering::Greater => {
                    let native_token = NativeToken::new(token_id, native_token_amount_acc - burn_token_amount)?;
                    let output = add_native_token_to_output(&input_and_output.2, native_token)?;
                    outputs.push(output);
                    break;
                }
            }
        }

        Ok((custom_inputs, outputs))
    }
}

fn replace_output_native_tokens(
    output: &Output,
    native_tokens: impl IntoIterator<Item = NativeToken>,
) -> crate::Result<Output> {
    let output = match &output {
        Output::Alias(alias_output) => {
            let alias_output = AliasOutputBuilder::from(alias_output)
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

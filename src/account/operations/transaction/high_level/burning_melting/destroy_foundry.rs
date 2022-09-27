// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::output::{AliasId, AliasOutputBuilder, FoundryId, NativeTokensBuilder, Output, TokenScheme};

use crate::{
    account::{handle::AccountHandle, operations::transaction::Transaction, types::OutputData, TransactionOptions},
    Error,
};

impl AccountHandle {
    /// Function to destroy a foundry output with a circulating supply of 0.
    /// Native tokens in the foundry (minted by other foundries) will be transacted to the controlling alias
    pub async fn destroy_foundry(
        &self,
        foundry_id: FoundryId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] destroy_foundry");

        let token_supply = self.client.get_token_supply()?;
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
                options.allow_burning = true;
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                allow_burning: true,
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
                    .finish(token_supply)?;

                vec![Output::Alias(alias_output)]
            }
            _ => unreachable!("We checked if it's an alias output before"),
        };

        self.send(outputs, options).await
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

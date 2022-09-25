// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::output::{
    AliasOutputBuilder, FoundryOutputBuilder, Output, SimpleTokenScheme, TokenId, TokenScheme,
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        handle::AccountHandle, operations::transaction::high_level::minting::mint_native_token::MintTokenTransaction,
        TransactionOptions,
    },
    Error,
};

/// Address and foundry data for `mint_native_token()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncreaseNativeTokenSupplyOptions {}

/// Dto for IncreaseNativeTokenSupplyOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncreaseNativeTokenSupplyOptionsDto {}

impl TryFrom<&IncreaseNativeTokenSupplyOptionsDto> for IncreaseNativeTokenSupplyOptions {
    type Error = crate::Error;

    fn try_from(_value: &IncreaseNativeTokenSupplyOptionsDto) -> crate::Result<Self> {
        Ok(Self {})
    }
}

impl AccountHandle {
    /// Function to mint more native tokens when the max supply isn't reached yet. The foundry needs to be controlled by
    /// this account. Address needs to be Bech32 encoded. This will not change the max supply.
    /// ```ignore
    /// let tx = account_handle.increase_native_token_supply(
    ///             TokenId::from_str("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?,
    ///             U256::from(100),
    ///             native_token_options,
    ///             None
    ///         ).await?;
    /// println!("Transaction created: {}", tx.transaction_id);
    /// if let Some(block_id) = tx.block_id {
    ///     println!("Block sent: {}", block_id);
    /// }
    /// ```
    pub async fn increase_native_token_supply(
        &self,
        token_id: TokenId,
        mint_amount: U256,
        _increase_native_token_supply_options: Option<IncreaseNativeTokenSupplyOptions>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<MintTokenTransaction> {
        log::debug!("[TRANSACTION] increase_native_token_supply");

        let account = self.read().await;
        let token_supply = self.client.get_token_supply()?;
        let existing_foundry_output = account.unspent_outputs().values().into_iter().find(|output_data| {
            if let Output::Foundry(output) = &output_data.output {
                TokenId::new(*output.id()) == token_id
            } else {
                false
            }
        });

        let existing_foundry_output = existing_foundry_output
            .ok_or_else(|| Error::MintingFailed(format!("foundry output {} is not available", token_id)))?
            .clone();

        let existing_alias_output = if let Output::Foundry(foundry_output) = &existing_foundry_output.output {
            let TokenScheme::Simple(token_scheme) = foundry_output.token_scheme();
            // Check if we can mint the provided amount without exceeding the maximum_supply
            if token_scheme.maximum_supply() - token_scheme.circulating_supply() < mint_amount {
                return Err(Error::MintingFailed(format!(
                    "minting additional {} tokens would exceed the maximum supply: {}",
                    mint_amount,
                    token_scheme.maximum_supply()
                )));
            }

            // Get the alias output that controls the foundry output
            let existing_alias_output = account.unspent_outputs().values().into_iter().find(|output_data| {
                if let Output::Alias(output) = &output_data.output {
                    output.alias_id().or_from_output_id(output_data.output_id) == **foundry_output.alias_address()
                } else {
                    false
                }
            });
            existing_alias_output
                .ok_or_else(|| Error::MintingFailed("alias output is not available".to_string()))?
                .clone()
        } else {
            return Err(Error::MintingFailed("alias output is not available".to_string()));
        };

        drop(account);

        let alias_output = if let Output::Alias(alias_output) = existing_alias_output.output {
            alias_output
        } else {
            unreachable!("We checked if it's an alias output before")
        };
        let foundry_output = if let Output::Foundry(foundry_output) = existing_foundry_output.output {
            foundry_output
        } else {
            unreachable!("We checked if it's an foundry output before")
        };

        // Create the next alias output with the same data, just updated state_index
        let new_alias_output_builder =
            AliasOutputBuilder::from(&alias_output).with_state_index(alias_output.state_index() + 1);

        // Create next foundry output with minted native tokens

        let TokenScheme::Simple(token_scheme) = foundry_output.token_scheme();

        let updated_token_scheme = TokenScheme::Simple(SimpleTokenScheme::new(
            token_scheme.minted_tokens() + mint_amount,
            *token_scheme.melted_tokens(),
            *token_scheme.maximum_supply(),
        )?);

        let new_foundry_output_builder =
            FoundryOutputBuilder::from(&foundry_output).with_token_scheme(updated_token_scheme);

        let outputs = vec![
            new_alias_output_builder.finish_output(token_supply)?,
            new_foundry_output_builder.finish_output(token_supply)?,
            // Native Tokens will be added automatically in the remainder output in try_select_inputs()
        ];

        self.send(outputs, options)
            .await
            .map(|transaction| MintTokenTransaction { token_id, transaction })
    }
}

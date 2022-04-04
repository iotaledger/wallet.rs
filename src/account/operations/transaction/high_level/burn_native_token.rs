// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::TransferResult,
        types::AccountAddress,
        TransferOptions,
    },
    Error,
};

use iota_client::bee_message::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::{
            GovernorAddressUnlockCondition, ImmutableAliasAddressUnlockCondition,
            StateControllerAddressUnlockCondition, UnlockCondition,
        },
        AliasOutputBuilder, FoundryOutputBuilder, NativeToken, Output, SimpleTokenScheme, TokenId, TokenScheme,
    },
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address, token ID and amount for `burn_native_token()`
#[serde(rename_all = "camelCase")]
pub struct BurnNativeTokenOptions {
    /// Bech32 encoded address. Needs to be an account address. Default will use the
    /// first address of the account
    pub account_address: Option<String>,
    /// Native token
    pub native_token: (TokenId, U256),
}

impl AccountHandle {
    fn contains_bech32_address(address: &str, account_addresses: &[AccountAddress]) -> crate::Result<Address> {
        let (_bech32_hrp, address) = Address::try_from_bech32(address)?;
        if account_addresses
            .binary_search_by_key(&address, |address| address.address.inner)
            .is_err()
        {
            return Err(Error::AddressNotFoundInAccount);
        }
        Ok(address)
    }

    /// Function to burn native tokens with foundry
    pub async fn burn_native_token(
        &self,
        burn_native_token_options: BurnNativeTokenOptions,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] burn_native_token");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let account_addresses = self.list_addresses().await?;
        // the address needs to be from the account, because for the minting we need to sign transactions from it
        let controller_address = match &burn_native_token_options.account_address {
            Some(address) => AccountHandle::contains_bech32_address(address, &account_addresses)?,
            None => {
                account_addresses
                    .first()
                    // todo other error message
                    .ok_or(Error::FailedToGetRemainder)?
                    .address
                    .inner
            }
        };

        let token_id = burn_native_token_options.native_token.0;
        let burn_token_amount = burn_native_token_options.native_token.1;

        let foundry_id = token_id.foundry_id();
        let alias_id = *foundry_id.alias_address().alias_id();

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
                Output::Foundry(output) => existing_foundry_output = Some(output),
                _ => unreachable!("We checked if it's an alias or foundry output before"),
            });

        let existing_alias_output_data = existing_alias_output_data
            .ok_or_else(|| Error::BurningFailed("No alias output available".to_string()))?
            .clone();

        let existing_foundry_output = existing_foundry_output
            .ok_or_else(|| Error::BurningFailed("No foundry output available".to_string()))?
            .clone();

        drop(account);

        if let Output::Alias(alias_output) = &existing_alias_output_data.output {
            // Create the new alias output with the same feature blocks, just updated state_index
            let mut new_alias_output_builder = AliasOutputBuilder::new_with_amount(existing_alias_output_data.amount, alias_id)?
                .with_state_index(alias_output.state_index() + 1)
                .with_foundry_counter(alias_output.foundry_counter())
                .add_unlock_condition(UnlockCondition::StateControllerAddress(
                    StateControllerAddressUnlockCondition::new(controller_address),
                ))
                .add_unlock_condition(UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(
                    controller_address,
                )));
            for feature_block in alias_output.feature_blocks().iter() {
                new_alias_output_builder = new_alias_output_builder.add_feature_block(feature_block.clone());
            }
            for immutable_feature_block in alias_output.immutable_feature_blocks().iter() {
                new_alias_output_builder =
                    new_alias_output_builder.add_immutable_feature_block(immutable_feature_block.clone());
            }

            let TokenScheme::Simple(foundry_simple_ts) = existing_foundry_output.token_scheme();
            let outputs = vec![
                Output::Alias(new_alias_output_builder.finish()?),
                Output::Foundry(
                    FoundryOutputBuilder::new_with_minimum_storage_deposit(
                        byte_cost_config,
                        foundry_id.serial_number(),
                        token_id.token_tag(),
                        TokenScheme::Simple(SimpleTokenScheme::new(
                            foundry_simple_ts.circulating_supply(),
                            foundry_simple_ts.melted_tokens() + burn_token_amount,
                            *foundry_simple_ts.maximum_supply(),
                        )?),
                    )?
                    .add_native_token(NativeToken::new(
                        token_id,
                        foundry_simple_ts.circulating_supply() - burn_token_amount,
                    )?)
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
}

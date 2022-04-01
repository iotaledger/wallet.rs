// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::{
            high_level::minimum_storage_deposit::{minimum_storage_deposit_alias, minimum_storage_deposit_foundry},
            TransferResult,
        },
        TransferOptions,
    },
    Error,
};

use iota_client::{
    api::input_selection::minimum_storage_deposit,
    bee_message::{
        address::{Address, AliasAddress},
        output::{
            unlock_condition::{
                AddressUnlockCondition, GovernorAddressUnlockCondition, ImmutableAliasAddressUnlockCondition,
                StateControllerAddressUnlockCondition, UnlockCondition,
            },
            AliasId, AliasOutputBuilder, BasicOutputBuilder, FoundryId, FoundryOutputBuilder, NativeToken, Output,
            SimpleTokenScheme, TokenId, TokenScheme, TokenTag,
        },
    },
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address and nft for `mint_native_token()`
pub struct NativeTokenOptions {
    /// Bech32 encoded address. Needs to be an account address. Default will use the
    /// first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
    /// Token tag
    #[serde(rename = "tokenTag")]
    pub token_tag: TokenTag,
    /// Circulating supply
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: U256,
    /// Maximum supply
    #[serde(rename = "maxiumSupply")]
    pub maxium_supply: U256,
}

impl AccountHandle {
    /// Function to mint native tokens
    /// This happens in a two step process:
    /// 1. Create or get an existing alias output
    /// 2. Create a new foundry output with native tokens minted to the account address
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let native_token_options = NativeTokenOptions {
    ///     account_address: None,
    ///     token_tag: TokenTag::new([0u8; 12]),
    ///     circulating_supply: U256::from(100),
    ///     maxium_supply: U256::from(100),
    /// };
    ///
    /// let res = account_handle.mint_native_token(native_token_options, None,).await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn mint_native_token(
        &self,
        native_token_options: NativeTokenOptions,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] mint_native_token");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let account_addresses = self.list_addresses().await?;
        // the address needs to be from the account, because for the minting we need to sign transactions from it
        let controller_address = match &native_token_options.account_address {
            Some(address) => {
                let (_bech32_hrp, address) = Address::try_from_bech32(&address)?;
                if account_addresses
                    .binary_search_by_key(&address, |address| address.address.inner)
                    .is_err()
                {
                    return Err(Error::AddressNotFoundInAccount);
                }
                address
            }
            None => {
                account_addresses
                    .first()
                    // todo other error message
                    .ok_or(Error::FailedToGetRemainder)?
                    .address
                    .inner
            }
        };

        let alias_id = self
            .get_or_create_alias_output(controller_address, options.clone())
            .await?;

        let account = self.read().await;
        let existing_alias_output = account.unspent_outputs().values().into_iter().find(|output_data| {
            if let Output::Alias(output) = &output_data.output {
                output.alias_id().or_from_output_id(output_data.output_id) == alias_id
            } else {
                false
            }
        });
        let existing_alias_output = existing_alias_output
            .ok_or_else(|| Error::MintingFailed("No alias output available".to_string()))?
            .clone();
        drop(account);

        if let Output::Alias(alias_output) = &existing_alias_output.output {
            // create foundry output with minted native tokens
            let foundry_id = FoundryId::build(
                &AliasAddress::new(alias_id),
                alias_output.foundry_counter() + 1,
                SimpleTokenScheme::KIND,
            );
            let token_id = TokenId::build(&foundry_id, &native_token_options.token_tag);

            // Create the new alias output with the same feature blocks, just updated state_index and foundry_counter
            let mut new_alias_output_builder = AliasOutputBuilder::new(existing_alias_output.amount, alias_id)?
                .with_state_index(alias_output.state_index() + 1)
                .with_foundry_counter(alias_output.foundry_counter() + 1)
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

            // todo: clean this up because it's only required for the stroage deposit calculation
            let mut native_tokens_for_storage_deposit = std::collections::HashMap::new();
            native_tokens_for_storage_deposit.insert(token_id, native_token_options.circulating_supply);

            let outputs = vec![
                Output::Alias(new_alias_output_builder.finish()?),
                Output::Foundry(
                    FoundryOutputBuilder::new(
                        minimum_storage_deposit_foundry(&byte_cost_config)?,
                        alias_output.foundry_counter() + 1,
                        native_token_options.token_tag,
                        TokenScheme::Simple(SimpleTokenScheme::new(
                            native_token_options.circulating_supply,
                            U256::from(0),
                            native_token_options.maxium_supply,
                        )?),
                    )?
                    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
                        ImmutableAliasAddressUnlockCondition::new(AliasAddress::from(alias_id)),
                    ))
                    .finish()?,
                ),
                Output::Basic(
                    BasicOutputBuilder::new(minimum_storage_deposit(
                        &byte_cost_config,
                        &controller_address,
                        &Some(native_tokens_for_storage_deposit),
                    )?)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        controller_address,
                    )))
                    .add_native_token(NativeToken::new(token_id, native_token_options.circulating_supply)?)
                    .finish()?,
                ),
            ];
            self.send(outputs, options).await
        } else {
            unreachable!("We checked if it's an alias output before")
        }
    }

    // Get an existing alias output or create a new one
    pub(crate) async fn get_or_create_alias_output(
        &self,
        controller_address: Address,
        options: Option<TransferOptions>,
    ) -> crate::Result<AliasId> {
        log::debug!("[TRANSFER] get_or_create_alias_output");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let account = self.read().await;
        let existing_alias_output = account
            .unspent_outputs()
            .values()
            .into_iter()
            .find(|output_data| matches!(&output_data.output, Output::Alias(output)));
        match existing_alias_output {
            Some(output_data) => {
                if let Output::Alias(alias_output) = &output_data.output {
                    let alias_id = alias_output.alias_id().or_from_output_id(output_data.output_id);
                    Ok(alias_id)
                } else {
                    unreachable!("We checked if it's an alias output before")
                }
            }
            // Create a new alias output
            None => {
                drop(account);
                let amount = minimum_storage_deposit_alias(&byte_cost_config, &controller_address)?;
                let outputs = vec![Output::Alias(
                    AliasOutputBuilder::new(amount, AliasId::from([0; 20]))?
                        .with_state_index(0)
                        .with_foundry_counter(0)
                        .add_unlock_condition(UnlockCondition::StateControllerAddress(
                            StateControllerAddressUnlockCondition::new(controller_address),
                        ))
                        .add_unlock_condition(UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(
                            controller_address,
                        )))
                        .finish()?,
                )];
                let transfer_result = self.send(outputs, options).await?;
                log::debug!("[TRANSFER] sent alias output");
                if let Some(message_id) = transfer_result.message_id {
                    self.client.retry_until_included(&message_id, None, None).await?;
                } else {
                    self.sync_pending_transactions().await?;
                }

                for i in 0..10 {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    self.sync_pending_transactions().await?;
                    let balance = self.sync(None).await?;
                    if !balance.aliases.is_empty() {
                        return Ok(balance.aliases[0]);
                    }
                }

                Err(Error::MintingFailed("Alias output creation took too long".to_string()))
            }
        }
    }
}

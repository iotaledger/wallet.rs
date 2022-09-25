// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    address::AliasAddress,
    dto::U256Dto,
    output::{
        dto::AliasIdDto,
        feature::{Feature, MetadataFeature},
        unlock_condition::{ImmutableAliasAddressUnlockCondition, UnlockCondition},
        AliasId, AliasOutputBuilder, FoundryId, FoundryOutputBuilder, Output, SimpleTokenScheme, TokenId, TokenScheme,
    },
    DtoError,
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::account::{
    handle::AccountHandle,
    types::{Transaction, TransactionDto},
    TransactionOptions,
};

/// Address and foundry data for `mint_native_token()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTokenOptions {
    /// The alias id which should be used to create the foundry.
    #[serde(rename = "aliasId")]
    pub alias_id: Option<AliasId>,
    /// Circulating supply
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: U256,
    /// Maximum supply
    #[serde(rename = "maximumSupply")]
    pub maximum_supply: U256,
    /// Foundry metadata
    #[serde(rename = "foundryMetadata")]
    pub foundry_metadata: Option<Vec<u8>>,
}

/// Dto for NativeTokenOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTokenOptionsDto {
    /// The alias id which should be used to create the foundry.
    #[serde(rename = "aliasId")]
    pub alias_id: Option<AliasIdDto>,
    /// Circulating supply
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: U256Dto,
    /// Maximum supply
    #[serde(rename = "maximumSupply")]
    pub maximum_supply: U256Dto,
    /// Foundry metadata, hex encoded bytes
    #[serde(rename = "foundryMetadata")]
    pub foundry_metadata: Option<String>,
}

impl TryFrom<&NativeTokenOptionsDto> for NativeTokenOptions {
    type Error = crate::Error;

    fn try_from(value: &NativeTokenOptionsDto) -> crate::Result<Self> {
        Ok(Self {
            alias_id: match &value.alias_id {
                Some(alias_id) => Some(AliasId::try_from(alias_id)?),
                None => None,
            },
            circulating_supply: U256::try_from(&value.circulating_supply)
                .map_err(|_| DtoError::InvalidField("circulating_supply"))?,
            maximum_supply: U256::try_from(&value.maximum_supply)
                .map_err(|_| DtoError::InvalidField("maximum_supply"))?,
            foundry_metadata: match &value.foundry_metadata {
                Some(metadata) => {
                    Some(prefix_hex::decode(metadata).map_err(|_| DtoError::InvalidField("foundry_metadata"))?)
                }
                None => None,
            },
        })
    }
}

/// The result of a minting native token transaction
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintTokenTransaction {
    pub token_id: TokenId,
    pub transaction: Transaction,
}

/// Dto for MintTokenTransaction
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintTokenTransactionDto {
    pub token_id: TokenId,
    pub transaction: TransactionDto,
}

impl From<&MintTokenTransaction> for MintTokenTransactionDto {
    fn from(value: &MintTokenTransaction) -> Self {
        Self {
            token_id: value.token_id,
            transaction: TransactionDto::from(&value.transaction),
        }
    }
}

impl AccountHandle {
    /// Function to create a new foundry output with minted native tokens.
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let native_token_options = NativeTokenOptions {
    ///     alias_id: None,
    ///     circulating_supply: U256::from(100),
    ///     maximum_supply: U256::from(100),
    ///     foundry_metadata: None
    /// };
    ///
    /// let tx = account_handle.mint_native_token(native_token_options, None,).await?;
    /// println!("Transaction created: {}", tx.transaction_id);
    /// if let Some(block_id) = tx.block_id {
    ///     println!("Block sent: {}", block_id);
    /// }
    /// ```
    pub async fn mint_native_token(
        &self,
        native_token_options: NativeTokenOptions,
        options: Option<TransactionOptions>,
    ) -> crate::Result<MintTokenTransaction> {
        log::debug!("[TRANSACTION] mint_native_token");
        let rent_structure = self.client.get_rent_structure()?;

        let (alias_id, alias_output) = self
            .get_alias_output(native_token_options.alias_id)
            .await
            .ok_or_else(|| crate::Error::MintingFailed("Missing alias output".to_string()))?;

        if let Output::Alias(alias_output) = &alias_output.output {
            // Create the new alias output with the same feature blocks, just updated state_index and foundry_counter
            let new_alias_output_builder = AliasOutputBuilder::from(alias_output)
                .with_alias_id(alias_id)
                .with_state_index(alias_output.state_index() + 1)
                .with_foundry_counter(alias_output.foundry_counter() + 1);

            // create foundry output with minted native tokens
            let foundry_id = FoundryId::build(
                &AliasAddress::new(alias_id),
                alias_output.foundry_counter() + 1,
                SimpleTokenScheme::KIND,
            );
            let token_id = TokenId::from(foundry_id);

            let outputs = vec![
                new_alias_output_builder.finish_output()?,
                {
                    let mut foundry_builder = FoundryOutputBuilder::new_with_minimum_storage_deposit(
                        rent_structure.clone(),
                        alias_output.foundry_counter() + 1,
                        TokenScheme::Simple(SimpleTokenScheme::new(
                            native_token_options.circulating_supply,
                            U256::from(0u8),
                            native_token_options.maximum_supply,
                        )?),
                    )?
                    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
                        ImmutableAliasAddressUnlockCondition::new(AliasAddress::from(alias_id)),
                    ));

                    if let Some(foundry_metadata) = native_token_options.foundry_metadata {
                        foundry_builder = foundry_builder
                            .add_immutable_feature(Feature::Metadata(MetadataFeature::new(foundry_metadata)?))
                    }

                    foundry_builder.finish_output()?
                }, // Native Tokens will be added automatically in the remainder output in try_select_inputs()
            ];
            self.send(outputs, options)
                .await
                .map(|transaction| MintTokenTransaction { token_id, transaction })
        } else {
            unreachable!("We checked if it's an alias output before")
        }
    }
}

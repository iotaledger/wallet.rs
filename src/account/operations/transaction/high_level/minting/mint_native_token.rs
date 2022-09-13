// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    address::{Address, AliasAddress},
    dto::U256Dto,
    output::{
        feature::{Feature, MetadataFeature},
        unlock_condition::{
            GovernorAddressUnlockCondition, ImmutableAliasAddressUnlockCondition,
            StateControllerAddressUnlockCondition, UnlockCondition,
        },
        AliasId, AliasOutputBuilder, FoundryId, FoundryOutputBuilder, Output, SimpleTokenScheme, TokenId, TokenScheme,
    },
    DtoError,
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        handle::AccountHandle,
        types::{Transaction, TransactionDto},
        TransactionOptions,
    },
    Error,
};

/// Address and foundry data for `mint_native_token()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTokenOptions {
    /// Bech32 encoded address. Needs to be an account address. Default will use the first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
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
    /// Bech32 encoded address. Needs to be an account address. Default will use the first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
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
            account_address: value.account_address.clone(),
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
        let rent_structure = self.client.get_rent_structure().await?;

        let account_addresses = self.list_addresses().await?;
        // the address needs to be from the account, because for the minting we need to sign transactions from it
        let controller_address = match &native_token_options.account_address {
            Some(bech32_address) => {
                let (_bech32_hrp, address) = Address::try_from_bech32(&bech32_address)?;
                if !account_addresses.iter().any(|addr| addr.address.inner == address) {
                    return Err(Error::AddressNotFoundInAccount(bech32_address.to_string()));
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
            .ok_or_else(|| Error::MintingFailed("no alias output available".to_string()))?
            .clone();
        drop(account);

        if let Output::Alias(alias_output) = &existing_alias_output.output {
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

    /// Get an existing alias output or create a new one
    pub(crate) async fn get_or_create_alias_output(
        &self,
        controller_address: Address,
        options: Option<TransactionOptions>,
    ) -> crate::Result<AliasId> {
        log::debug!("[TRANSACTION] get_or_create_alias_output");
        let rent_structure = self.client.get_rent_structure().await?;

        let account = self.read().await;
        let existing_alias_output = account
            .unspent_outputs()
            .values()
            .into_iter()
            .find(|output_data| matches!(&output_data.output, Output::Alias(_output)));
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
                let outputs = vec![
                    AliasOutputBuilder::new_with_minimum_storage_deposit(rent_structure, AliasId::null())?
                        .with_state_index(0)
                        .with_foundry_counter(0)
                        .add_unlock_condition(UnlockCondition::StateControllerAddress(
                            StateControllerAddressUnlockCondition::new(controller_address),
                        ))
                        .add_unlock_condition(UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(
                            controller_address,
                        )))
                        .finish_output()?,
                ];
                let transaction = self.send(outputs, options).await?;

                log::debug!("[TRANSACTION] sent alias output");
                if let Some(block_id) = transaction.block_id {
                    self.client.retry_until_included(&block_id, None, None).await?;
                }
                // Try to get the transaction confirmed
                for _ in 0..10 {
                    let balance = self.sync(None).await?;
                    if !balance.aliases.is_empty() {
                        return Ok(balance.aliases[0]);
                    }
                    self.sync_pending_transactions().await?;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(Error::MintingFailed("alias output creation took too long".to_string()))
            }
        }
    }
}

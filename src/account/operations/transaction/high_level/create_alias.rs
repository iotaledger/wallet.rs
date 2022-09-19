// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    block::{
        address::Address,
        output::{
            feature::{Feature, MetadataFeature},
            unlock_condition::{
                GovernorAddressUnlockCondition, StateControllerAddressUnlockCondition, UnlockCondition,
            },
            AliasId, AliasOutputBuilder, Output,
        },
        DtoError,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{handle::AccountHandle, types::Transaction, OutputData, TransactionOptions},
    Error,
};

/// Address and nft for `send_nft()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasOutputOptions {
    /// Bech32 encoded address. Needs to be an account address. Default will use the first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
    /// Immutable nft metadata
    #[serde(rename = "immutableMetadata")]
    pub immutable_metadata: Option<Vec<u8>>,
    /// Nft metadata
    pub metadata: Option<Vec<u8>>,
}

/// Dto for NftOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasOutputOptionsDto {
    /// Bech32 encoded address to which the Nft will be minted. Default will use the
    /// first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
    /// Immutable alias metadata, hex encoded bytes
    #[serde(rename = "immutableMetadata")]
    pub immutable_metadata: Option<String>,
    /// Alias metadata, hex encoded bytes
    pub metadata: Option<String>,
}

impl TryFrom<&AliasOutputOptionsDto> for AliasOutputOptions {
    type Error = crate::Error;

    fn try_from(value: &AliasOutputOptionsDto) -> crate::Result<Self> {
        Ok(Self {
            account_address: value.account_address.clone(),
            immutable_metadata: match &value.immutable_metadata {
                Some(metadata) => {
                    Some(prefix_hex::decode(metadata).map_err(|_| DtoError::InvalidField("immutable_metadata"))?)
                }
                None => None,
            },
            metadata: match &value.metadata {
                Some(metadata) => Some(prefix_hex::decode(metadata).map_err(|_| DtoError::InvalidField("metadata"))?),
                None => None,
            },
        })
    }
}

impl AccountHandle {
    /// Function to create an alias output.
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let alias_options = AliasOutputOptions {
    ///     address: Some("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string()),
    ///     immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    ///     metadata: Some(b"some nft metadata".to_vec()),
    /// };
    ///
    /// let transaction = account.create_alias_output(alias_options, None).await?;
    /// println!(
    ///     "Transaction: {} Block sent: http://localhost:14265/api/core/v2/blocks/{}",
    ///     transaction.transaction_id,
    ///     transaction.block_id.expect("no block created yet")
    /// );
    /// ```
    pub async fn create_alias_output(
        &self,
        alias_output_options: AliasOutputOptions,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        let prepared_transaction = self.prepare_create_alias_output(alias_output_options, options).await?;
        self.sign_and_submit_transaction(prepared_transaction).await
    }

    pub(crate) async fn prepare_create_alias_output(
        &self,
        alias_output_options: AliasOutputOptions,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] get_or_create_alias_output");
        let rent_structure = self.client.get_rent_structure().await?;

        let account_addresses = self.list_addresses().await?;
        // the address needs to be from the account, because for the minting we need to sign transactions from it
        let controller_address = match &alias_output_options.account_address {
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

        let mut alias_output_builder =
            AliasOutputBuilder::new_with_minimum_storage_deposit(rent_structure, AliasId::null())?
                .with_state_index(0)
                .with_foundry_counter(0)
                .add_unlock_condition(UnlockCondition::StateControllerAddress(
                    StateControllerAddressUnlockCondition::new(controller_address),
                ))
                .add_unlock_condition(UnlockCondition::GovernorAddress(GovernorAddressUnlockCondition::new(
                    controller_address,
                )));
        if let Some(immutable_metadata) = alias_output_options.immutable_metadata {
            alias_output_builder = alias_output_builder
                .add_immutable_feature(Feature::Metadata(MetadataFeature::new(immutable_metadata)?));
        };
        if let Some(metadata) = alias_output_options.metadata {
            alias_output_builder = alias_output_builder.add_feature(Feature::Metadata(MetadataFeature::new(metadata)?));
        };

        let outputs = vec![alias_output_builder.finish_output()?];

        self.prepare_transaction(outputs, options).await
    }

    /// Get an existing alias output
    pub(crate) async fn get_alias_output(&self, alias_id: Option<AliasId>) -> Option<(AliasId, OutputData)> {
        log::debug!("[get_alias_output]");
        self.read()
            .await
            .unspent_outputs()
            .values()
            .into_iter()
            .find_map(|output_data| match &output_data.output {
                Output::Alias(alias_output) => {
                    let output_alias_id = alias_output.alias_id().or_from_output_id(output_data.output_id);
                    if let Some(alias_id) = alias_id {
                        if output_alias_id == alias_id {
                            Some((output_alias_id, output_data.clone()))
                        } else {
                            None
                        }
                    } else {
                        Some((output_alias_id, output_data.clone()))
                    }
                }
                _ => None,
            })
    }
}

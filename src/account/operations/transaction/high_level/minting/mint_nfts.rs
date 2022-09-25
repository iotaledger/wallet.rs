// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    block::{
        address::Address,
        output::{
            feature::{Feature, MetadataFeature},
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            NftId, NftOutputBuilder,
        },
        DtoError,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{handle::AccountHandle, operations::transaction::Transaction, TransactionOptions},
    Error,
};

/// Address and nft for `send_nft()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftOptions {
    /// Bech32 encoded address to which the Nft will be minted. Default will use the
    /// first address of the account
    pub address: Option<String>,
    /// Immutable nft metadata
    #[serde(rename = "immutableMetadata")]
    pub immutable_metadata: Option<Vec<u8>>,
    /// Nft metadata
    pub metadata: Option<Vec<u8>>,
}

/// Dto for NftOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftOptionsDto {
    /// Bech32 encoded address to which the Nft will be minted. Default will use the
    /// first address of the account
    pub address: Option<String>,
    /// Immutable nft metadata, hex encoded bytes
    #[serde(rename = "immutableMetadata")]
    pub immutable_metadata: Option<String>,
    /// Nft metadata, hex encoded bytes
    pub metadata: Option<String>,
}

impl TryFrom<&NftOptionsDto> for NftOptions {
    type Error = crate::Error;

    fn try_from(value: &NftOptionsDto) -> crate::Result<Self> {
        Ok(Self {
            address: value.address.clone(),
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
    /// Function to mint nfts.
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let nft_id: [u8; 38] =
    ///     prefix_hex::decode("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?
    ///         .try_into()
    ///         .unwrap();
    /// let nft_options = vec![NftOptions {
    ///     address: Some("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string()),
    ///     immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    ///     metadata: Some(b"some nft metadata".to_vec()),
    /// }];
    ///
    /// let transaction = account.mint_nfts(nft_options, None).await?;
    /// println!(
    ///     "Transaction: {} Block sent: http://localhost:14265/api/core/v2/blocks/{}",
    ///     transaction.transaction_id,
    ///     transaction.block_id.expect("no block created yet")
    /// );
    /// ```
    pub async fn mint_nfts(
        &self,
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        let prepared_transaction = self.prepare_mint_nfts(nfts_options, options).await?;
        self.sign_and_submit_transaction(prepared_transaction).await
    }

    /// Function to prepare the transaction for
    /// [AccountHandle.mint_nfts()](crate::account::handle::AccountHandle.mint_nfts)
    async fn prepare_mint_nfts(
        &self,
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_mint_nfts");
        let rent_structure = self.client.get_rent_structure()?;
        let account_addresses = self.addresses().await?;
        let mut outputs = Vec::new();

        for nft_options in nfts_options {
            let address = match nft_options.address {
                Some(address) => Address::try_from_bech32(&address)?.1,
                // todo other error message
                None => {
                    account_addresses
                        .first()
                        .ok_or(Error::FailedToGetRemainder)?
                        .address
                        .inner
                }
            };

            // NftId needs to be set to 0 for the creation
            let mut nft_builder =
                NftOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone(), NftId::null())?
                    // Address which will own the nft
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)));
            if let Some(immutable_metadata) = nft_options.immutable_metadata {
                nft_builder =
                    nft_builder.add_immutable_feature(Feature::Metadata(MetadataFeature::new(immutable_metadata)?));
            };
            if let Some(metadata) = nft_options.metadata {
                nft_builder = nft_builder.add_feature(Feature::Metadata(MetadataFeature::new(metadata)?));
            };
            outputs.push(nft_builder.finish_output()?);
        }

        self.prepare_transaction(outputs, options).await
    }
}

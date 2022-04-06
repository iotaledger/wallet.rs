// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::{high_level::minimum_storage_deposit::minimum_storage_deposit_nft, TransferResult},
        TransferOptions,
    },
    Error,
};

use iota_client::bee_message::{
    address::Address,
    output::{
        feature_block::{FeatureBlock, MetadataFeatureBlock},
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        NftId, NftOutputBuilder, Output,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address and nft for `send_nft()`
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

impl AccountHandle {
    /// Function to mint nfts.
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let nft_id: [u8; 38] =
    ///     hex::decode("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?
    ///         .try_into()
    ///         .unwrap();
    /// let nft_options = vec![NftOptions {
    ///     address: Some("atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string()),
    ///     immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    ///     metadata: Some(b"some nft metadata".to_vec()),
    /// }];
    ///
    /// let transfer_result = account.mint_nfts(nft_options, None).await?;
    /// println!(
    ///     "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
    ///     transfer_result.transaction_id,
    ///     transfer_result.message_id.expect("No message created yet")
    /// );
    /// ```
    pub async fn mint_nfts(
        &self,
        nfts_options: Vec<NftOptions>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] mint_nfts");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let account_addresses = self.list_addresses().await?;

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
            let immutable_metadata = if let Some(immutable_metadata) = nft_options.immutable_metadata {
                Some(FeatureBlock::Metadata(MetadataFeatureBlock::new(immutable_metadata)?))
            } else {
                None
            };
            let metadata = if let Some(metadata) = nft_options.metadata {
                Some(FeatureBlock::Metadata(MetadataFeatureBlock::new(metadata)?))
            } else {
                None
            };

            let minimum_storage_deposit = minimum_storage_deposit_nft(
                &byte_cost_config,
                &address,
                immutable_metadata.clone(),
                metadata.clone(),
            )?;
            // NftId needs to be set to 0 for the creation
            let mut nft_builder = NftOutputBuilder::new(minimum_storage_deposit, NftId::from([0; 20]))?
                // Address which will own the nft
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)));
            if let Some(immutable_metadata) = immutable_metadata {
                nft_builder = nft_builder.add_immutable_feature_block(immutable_metadata);
            }
            if let Some(metadata) = metadata {
                nft_builder = nft_builder.add_feature_block(metadata);
            }
            outputs.push(Output::Nft(nft_builder.finish()?));
        }
        self.send(outputs, options).await
    }
}

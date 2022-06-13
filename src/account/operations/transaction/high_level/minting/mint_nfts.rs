// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    bee_block::{
        address::Address,
        output::{
            feature::{Feature, MetadataFeature},
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            NftId, NftOutputBuilder,
        },
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
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
    ///     address: Some("rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string()),
    ///     immutable_metadata: Some(b"some immutable nft metadata".to_vec()),
    ///     metadata: Some(b"some nft metadata".to_vec()),
    /// }];
    ///
    /// let transaction_result = account.mint_nfts(nft_options, None).await?;
    /// println!(
    ///     "Transaction: {} Block sent: http://localhost:14265/api/v2/blocks/{}",
    ///     transaction_result.transaction_id,
    ///     transaction_result.block_id.expect("No block created yet")
    /// );
    /// ```
    pub async fn mint_nfts(
        &self,
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        let prepared_trasacton = self.prepare_mint_nfts(nfts_options, options).await?;
        self.sign_and_submit_transaction(prepared_trasacton).await
    }

    /// Function to prepare the transaction for
    /// [AccountHandle.mint_nfts()](crate::account::handle::AccountHandle.mint_nfts)
    async fn prepare_mint_nfts(
        &self,
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_mint_nfts");
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
                Some(Feature::Metadata(MetadataFeature::new(immutable_metadata)?))
            } else {
                None
            };
            let metadata = if let Some(metadata) = nft_options.metadata {
                Some(Feature::Metadata(MetadataFeature::new(metadata)?))
            } else {
                None
            };

            // NftId needs to be set to 0 for the creation
            let mut nft_builder =
                NftOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config.clone(), NftId::null())?
                    // Address which will own the nft
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)));
            if let Some(immutable_metadata) = immutable_metadata {
                nft_builder = nft_builder.add_immutable_feature(immutable_metadata);
            }
            if let Some(metadata) = metadata {
                nft_builder = nft_builder.add_feature(metadata);
            }
            outputs.push(nft_builder.finish_output()?);
        }

        self.sync_and_prepare_transaction(outputs, options).await
    }
}

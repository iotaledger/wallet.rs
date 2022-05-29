// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    bee_block::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            NftId, NftOutputBuilder, Output,
        },
    },
};
// use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions};

/// Address and nft for `send_nft()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressAndNftId {
    /// Bech32 encoded address
    pub address: String,
    /// Nft id
    #[serde(rename = "nftId")]
    pub nft_id: NftId,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a
    /// [`StorageDepositReturnUnlockCondition`](iota_client::bee_block::output::unlock_condition::
    /// StorageDepositReturnUnlockCondition) and [`ExpirationUnlockCondition`](iota_client::bee_block::output::
    /// unlock_condition::ExpirationUnlockCondition), so the storage deposit gets back to the sender and also that
    /// the sender gets access to the output again after a defined time (default 1 day), Calls
    /// [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy. Custom inputs will be replaced with the required nft inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = vec![AddressAndNftId {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     nft_id: NftId::from_str("04f9b54d488d2e83a6c90db08ae4b39651bbba8a")?,
    /// }];
    ///
    /// let transaction_result = account.send_nft(outputs, None).await?;
    ///
    /// println!(
    /// "Transaction: {} Block sent: http://localhost:14265/api/v2/blocks/{}",
    /// transaction_result.transaction_id,
    /// transaction_result.block_id.expect("No block created yet")
    /// );
    /// ```
    pub async fn send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        let prepared_trasacton = self.prepare_send_nft(addresses_nft_ids, options).await?;
        self.sign_and_submit_transaction(prepared_trasacton).await
    }

    /// Function to prepare the transaction for
    /// [AccountHandle.send_nft()](crate::account::handle::AccountHandle.send_nft)
    pub async fn prepare_send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_send_nft");

        let unspent_outputs = self.list_unspent_outputs().await?;

        let mut outputs = Vec::new();
        let mut custom_inputs = Vec::new();

        for address_and_nft_id in addresses_nft_ids {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_and_nft_id.address)?;
            // Find nft output from the inputs
            if let Some(nft_output_data) = unspent_outputs.iter().find(|o| {
                if let Output::Nft(nft_output) = &o.output {
                    address_and_nft_id.nft_id == nft_output.nft_id().or_from_output_id(o.output_id)
                } else {
                    false
                }
            }) {
                if let Output::Nft(nft_output) = &nft_output_data.output {
                    // build new output with same amount, nft_id, immutable/feature blocks and native tokens, just
                    // updated address unlock conditions
                    let mut nft_builder =
                        NftOutputBuilder::new_with_amount(nft_output.amount(), address_and_nft_id.nft_id)?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)));
                    for native_token in nft_output.native_tokens().iter() {
                        nft_builder = nft_builder.add_native_token(native_token.clone());
                    }
                    for feature in nft_output.features().iter() {
                        nft_builder = nft_builder.add_feature(feature.clone());
                    }
                    for immutable_feature in nft_output.immutable_features().iter() {
                        nft_builder = nft_builder.add_immutable_feature(immutable_feature.clone());
                    }
                    outputs.push(nft_builder.finish_output()?);
                    // Add custom input
                    custom_inputs.push(nft_output_data.output_id);
                }
            } else {
                return Err(crate::Error::NftNotFoundInUnspentOutputs);
            };
        }
        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };

        self.sync_and_prepare_transaction(outputs, options).await
    }
}

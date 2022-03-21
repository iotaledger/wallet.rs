// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        NftId, NftOutputBuilder, Output,
    },
};
// use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address and nft for `send_nft()`
pub struct AddressAndNftId {
    /// Bech32 encoded address
    pub address: String,
    /// Nft id
    #[serde(rename = "nftId")]
    pub nft_id: NftId,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a [StorageDepositReturnUnlockCondition] and
    /// [ExpirationUnlockCondition], so the storage deposit gets back to the sender and also that the sender gets access
    /// to the output again after a defined time (default 1 day),
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy. Custom inputs will be replaced with the required nft inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = vec![AddressAndNftId {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     nft_id: NftId::from_str("04f9b54d488d2e83a6c90db08ae4b39651bbba8a")?,
    /// }];
    ///
    /// let transfer_result = account.send_nft(outputs, None).await?;
    ///
    /// println!(
    /// "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
    /// transfer_result.transaction_id,
    /// transfer_result.message_id.expect("No message created yet")
    /// );
    /// ```
    pub async fn send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let unspent_outputs = self.list_unspent_outputs().await?;
        let mut outputs = Vec::new();
        let mut custom_inputs = Vec::new();
        for address_and_nft_id in addresses_nft_ids {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_and_nft_id.address)?;
            // Find nft output from the inputs
            if let Some(nft_output_data) = unspent_outputs.iter().find(|o| {
                if let Output::Nft(nft_output) = &o.output {
                    // When the nft is minted, the nft_id contains only `0` bytes and we need to calculate the output id
                    // todo: replace with `.or_from_output_id(o.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                    let nft_id = if nft_output.nft_id().iter().all(|&b| b == 0) {
                        NftId::from(&o.output_id)
                    } else {
                        *nft_output.nft_id()
                    };
                    address_and_nft_id.nft_id == nft_id
                } else {
                    false
                }
            }) {
                if let Output::Nft(nft_output) = &nft_output_data.output {
                    // build new output with same amount, nft_id, immutable/feature blocks and native tokens, just
                    // updated address unlock conditions
                    let mut nft_builder = NftOutputBuilder::new(nft_output.amount(), address_and_nft_id.nft_id)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)));
                    for native_token in nft_output.native_tokens().iter() {
                        nft_builder = nft_builder.add_native_token(native_token.clone());
                    }
                    for feature_block in nft_output.feature_blocks().iter() {
                        nft_builder = nft_builder.add_feature_block(feature_block.clone());
                    }
                    for immutable_feature_block in nft_output.immutable_feature_blocks().iter() {
                        nft_builder = nft_builder.add_immutable_feature_block(immutable_feature_block.clone());
                    }
                    outputs.push(Output::Nft(nft_builder.finish()?));
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
            None => Some(TransferOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };
        self.send(outputs, options).await
    }
}

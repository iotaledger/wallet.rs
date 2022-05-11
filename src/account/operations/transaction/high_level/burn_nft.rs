// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

use iota_client::bee_message::{
    address::{Address, NftAddress},
    output::{BasicOutputBuilder, NftId, NftOutput, Output},
};

impl AccountHandle {
    /// Function to mint nft.
    pub async fn burn_nft(
        &self,
        nft_id: NftId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] burn_nft");

        let address = self.get_sweep_remainder_address(&options).await?;
        self.sweep_address_outputs(Address::Nft(NftAddress::new(nft_id)), &address)
            .await?;

        let (output_id, nft_output) = self.find_nft_output(nft_id).await?;
        let custom_inputs = vec![output_id];
        let outputs = vec![Self::nft_to_basic_output(&nft_output)?];

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

        self.send(outputs, options).await
    }

    fn nft_to_basic_output(nft_output: &NftOutput) -> crate::Result<Output> {
        Ok(Output::Basic(
            BasicOutputBuilder::new_with_amount(nft_output.amount())?
                .with_feature_blocks(nft_output.feature_blocks().clone())
                .with_unlock_conditions(nft_output.unlock_conditions().clone())
                .with_native_tokens(nft_output.native_tokens().clone())
                .finish()?,
        ))
    }
}

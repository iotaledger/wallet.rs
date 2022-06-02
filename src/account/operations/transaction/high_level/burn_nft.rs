// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_block::{
    address::{Address, NftAddress},
    output::{BasicOutputBuilder, NftId, Output, OutputId},
};

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
    Error,
};

impl AccountHandle {
    /// Function to burn nft.
    pub async fn burn_nft(
        &self,
        nft_id: NftId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] burn_nft");

        let address = self.get_sweep_remainder_address(&options).await?;
        self.sweep_address_outputs(Address::Nft(NftAddress::new(nft_id)), &address)
            .await?;

        let (output_id, basic_output) = self.output_id_and_basic_output_for_nft(nft_id).await?;
        let custom_inputs = vec![output_id];
        let outputs = vec![basic_output];

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

    async fn output_id_and_basic_output_for_nft(&self, nft_id: NftId) -> crate::Result<(OutputId, Output)> {
        let account = self.read().await;

        let (output_id, nft_output) = account
            .unspent_outputs()
            .iter()
            .find_map(|(&output_id, output_data)| match &output_data.output {
                Output::Nft(nft_output) => {
                    if nft_output.nft_id().or_from_output_id(output_id) == nft_id {
                        Some((output_id, nft_output))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or(Error::NftNotFoundInUnspentOutputs)?;

        let basic_output = Output::Basic(
            BasicOutputBuilder::new_with_amount(nft_output.amount())?
                .with_unlock_conditions(nft_output.unlock_conditions().clone())
                .with_native_tokens(nft_output.native_tokens().clone())
                .finish()?,
        );

        Ok((output_id, basic_output))
    }
}

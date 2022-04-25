// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
    Error,
};

use iota_client::bee_block::output::{BasicOutputBuilder, NftId, NftOutput, Output};

impl AccountHandle {
    /// Function to mint nft.
    pub async fn burn_nft(&self, nft_id: NftId, options: Option<TransactionOptions>) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] burn_nft");
        let account = self.read().await;

        let mut custom_inputs = Vec::new();
        let mut outputs = Vec::new();

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Nft(nft_output) => nft_output.nft_id().or_from_output_id(output_id) == nft_id,
                _ => false,
            })
            .ok_or(Error::NftNotFoundInUnspentOutputs)?;

        match &output_data.output {
            Output::Nft(nft_output) => {
                custom_inputs.push(*output_id);
                outputs.push(Self::nft_to_basic_output(nft_output)?);
            }
            _ => unreachable!("We already checked that it's an alias output"),
        }

        drop(account);

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

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    address::{Address, NftAddress},
    output::{unlock_condition::UnlockCondition, BasicOutputBuilder, NftId, Output, OutputId},
};

use crate::{
    account::{
        handle::AccountHandle,
        operations::{helpers::time::can_output_be_unlocked_now, transaction::Transaction},
        TransactionOptions,
    },
    Error,
};

impl AccountHandle {
    /// Function to burn an nft output.
    pub async fn burn_nft(&self, nft_id: NftId, options: Option<TransactionOptions>) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] burn_nft");

        let current_time = self.client().get_time_checked().await?;

        let addresses_with_unspent_outputs = self.list_addresses_with_unspent_outputs().await?;

        let mut owned_outputs = Vec::new();

        for output_data in self.list_unspent_outputs(None).await? {
            // Ignore outputs with a single [UnlockCondition], because then it's an
            // [AddressUnlockCondition] and we own it already without
            // further restrictions
            if output_data
                .output
                .unlock_conditions()
                .expect("output needs to have unlock_conditions")
                .len()
                != 1
            {
                if can_output_be_unlocked_now(
                    // We use the addresses with unspent outputs, because other addresses of the
                    // account without unspent outputs can't be related to this output
                    &addresses_with_unspent_outputs,
                    &[Address::Nft(NftAddress::new(nft_id))],
                    &output_data,
                    current_time,
                ) {
                    owned_outputs.push(output_data);
                }
            } else {
                owned_outputs.push(output_data);
            }
        }

        if !owned_outputs.is_empty() {
            return Err(Error::BurningOrMeltingFailed("nft still owns outputs".to_string()));
        }

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

    // Get the current output id for the nft and build a basic output with the amount, native tokens and
    // governor address from the nft output.
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

        let address_unlock_condition = nft_output
            .unlock_conditions()
            .address()
            .expect("nft output needs to have an address unlock condition")
            .clone();

        let basic_output = Output::Basic(
            BasicOutputBuilder::new_with_amount(nft_output.amount())?
                .add_unlock_condition(UnlockCondition::Address(address_unlock_condition))
                .with_native_tokens(nft_output.native_tokens().clone())
                .finish()?,
        );

        Ok((output_id, basic_output))
    }
}

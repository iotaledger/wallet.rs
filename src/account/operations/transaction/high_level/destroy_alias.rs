// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

use iota_client::bee_message::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::AddressUnlockCondition, AliasId, AliasOutput, BasicOutputBuilder, Output, UnlockCondition,
    },
};

impl AccountHandle {
    /// Function to destroy alias.
    pub async fn destroy_alias(
        &self,
        alias_id: AliasId,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        log::debug!("[TRANSFER] destroy_alias");

        let address = self.get_sweep_remainder_address(&options).await?;
        self.sweep_address_outputs(Address::Alias(AliasAddress::new(alias_id)), &address)
            .await
            .unwrap();

        let (output_id, alias_output) = self.find_alias_output(alias_id).await?;

        let (custom_inputs, outputs) = {
            let custom_inputs = vec![output_id];
            let outputs = vec![self.alias_to_basic_output(&alias_output)?];
            (custom_inputs, outputs)
        };

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

    fn alias_to_basic_output(&self, alias_output: &AliasOutput) -> crate::Result<Output> {
        Ok(Output::Basic(
            BasicOutputBuilder::new_with_amount(alias_output.amount())?
                .with_feature_blocks(alias_output.feature_blocks().clone())
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    *alias_output.governor_address(),
                )))
                .with_native_tokens(alias_output.native_tokens().clone())
                .finish()?,
        ))
    }
}

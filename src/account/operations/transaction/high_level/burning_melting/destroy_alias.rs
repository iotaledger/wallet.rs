// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_block::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::AddressUnlockCondition, AliasId, BasicOutputBuilder, Output, OutputId, UnlockCondition,
    },
};

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
    Error,
};

impl AccountHandle {
    /// Function to destroy an alias output. Outputs controlled by it will be sweeped before if they don't have a
    /// storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
    /// sent to the governor address.
    pub async fn destroy_alias(
        &self,
        alias_id: AliasId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] destroy_alias");

        let address = self.get_sweep_remainder_address(&options).await?;
        self.sweep_address_outputs(Address::Alias(AliasAddress::new(alias_id)), address)
            .await?;

        let (output_id, basic_output) = self.output_id_and_basic_output_for_alias(alias_id).await?;

        let (custom_inputs, outputs) = {
            let custom_inputs = vec![output_id];
            let outputs = vec![basic_output];
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

    // Get the current output id for the alias and build a basic output with the amount, native tokens and
    // governor address from the alias output.
    async fn output_id_and_basic_output_for_alias(&self, alias_id: AliasId) -> crate::Result<(OutputId, Output)> {
        let account = self.read().await;

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Alias(alias_output) => alias_output.alias_id().or_from_output_id(output_id) == alias_id,
                _ => false,
            })
            .ok_or_else(|| Error::BurningOrMeltingFailed("Alias output not found".to_string()))?;

        let alias_output = match &output_data.output {
            Output::Alias(alias_output) => alias_output,
            _ => unreachable!("We already checked that it's an alias output"),
        };

        let basic_output = Output::Basic(
            BasicOutputBuilder::new_with_amount(alias_output.amount())?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    *alias_output.governor_address(),
                )))
                .with_native_tokens(alias_output.native_tokens().clone())
                .finish()?,
        );

        Ok((*output_id, basic_output))
    }
}

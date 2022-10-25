// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::AddressUnlockCondition, AliasId, BasicOutputBuilder, Output, OutputId, UnlockCondition,
    },
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
    /// Function to destroy an alias output.
    pub async fn destroy_alias(
        &self,
        alias_id: AliasId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] destroy_alias");

        let current_time = self.client().get_time_checked()?;

        let mut owned_outputs = Vec::new();

        for output_data in self.unspent_outputs(None).await? {
            if can_output_be_unlocked_now(
                // Don't provide any addresses here, since we're only interested in outputs that can be unlocked by
                // the alias address
                &[],
                &[Address::Alias(AliasAddress::new(alias_id))],
                &output_data,
                current_time,
                true,
            )? {
                owned_outputs.push(output_data);
            }
        }

        if !owned_outputs.is_empty() {
            return Err(Error::BurningOrMeltingFailed(format!(
                "alias still owns outputs: {:?}",
                owned_outputs.iter().map(|o| o.output_id).collect::<Vec<OutputId>>()
            )));
        }

        let (output_id, basic_output) = self.output_id_and_basic_output_for_alias(alias_id).await?;

        let (custom_inputs, outputs) = {
            let custom_inputs = vec![output_id];
            let outputs = vec![basic_output];
            (custom_inputs, outputs)
        };

        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                options.allow_burning = true;
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                allow_burning: true,
                ..Default::default()
            }),
        };

        self.send(outputs, options).await
    }

    // Get the current output id for the alias and build a basic output with the amount, native tokens and
    // governor address from the alias output.
    async fn output_id_and_basic_output_for_alias(&self, alias_id: AliasId) -> crate::Result<(OutputId, Output)> {
        let account = self.read().await;
        let token_supply = self.client.get_token_supply()?;

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Alias(alias_output) => alias_output.alias_id().or_from_output_id(output_id) == alias_id,
                _ => false,
            })
            .ok_or_else(|| Error::BurningOrMeltingFailed("alias output not found".to_string()))?;

        let alias_output = match &output_data.output {
            Output::Alias(alias_output) => alias_output,
            _ => unreachable!("we already checked that it's an alias output"),
        };

        let basic_output = Output::Basic(
            BasicOutputBuilder::new_with_amount(alias_output.amount())?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                    *alias_output.governor_address(),
                )))
                .with_native_tokens(alias_output.native_tokens().clone())
                .finish(token_supply)?,
        );

        Ok((*output_id, basic_output))
    }
}

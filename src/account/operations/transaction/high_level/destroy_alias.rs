// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
    Error,
};

use iota_client::bee_message::{
    address::{Address, AliasAddress},
    output::{
        unlock_condition::AddressUnlockCondition, AliasId, AliasOutput, BasicOutputBuilder, FoundryId, Output,
        SimpleTokenScheme, UnlockCondition,
    },
};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasOptions {
    /// Alias id for output to be destroyed
    pub alias_id: AliasId,
    /// Whether to burn all controlled foundries or error out if any of the foundries cannot be found
    pub burn_foundries: bool,
    /// Burn native tokens in foundry, or transfer to alias output if false
    pub burn_native_token_remainder: bool,
}

impl Default for AliasOptions {
    fn default() -> Self {
        AliasOptions {
            alias_id: AliasId::new([0u8; AliasId::LENGTH]),
            burn_foundries: true,
            burn_native_token_remainder: true,
        }
    }
}

impl AccountHandle {
    /// Function to destroy alias.
    pub async fn destroy_alias(
        &self,
        alias_options: AliasOptions,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] destroy_alias");

        let address = self.get_sweep_remainder_address(&options).await?;
        self.sweep_address_outputs(Address::Alias(AliasAddress::new(alias_options.alias_id)), address)
            .await?;

        let (mut output_id, mut alias_output) = self.find_alias_output(alias_options.alias_id).await?;

        if alias_options.burn_foundries {
            let foundries = self.alias_foundries(alias_options.alias_id, alias_output.foundry_counter());
            if !foundries.is_empty() {
                let transfer_result = self
                    .burn_foundries(foundries, options.clone(), alias_options.burn_native_token_remainder)
                    .await?;

                match transfer_result.message_id {
                    Some(message_id) => {
                        let _ = self.client.retry_until_included(&message_id, None, None).await?;
                        let _ = self.sync(None).await?;
                        (output_id, alias_output) = self.find_alias_output(alias_options.alias_id).await?;
                    }
                    _ => return Err(Error::BurningFailed("Unable to burn foundries".to_string())),
                }
            }
        }

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

    fn alias_foundries(&self, alias_id: AliasId, foundry_counter: u32) -> HashSet<FoundryId> {
        let alias_address = AliasAddress::new(alias_id);
        let mut foundry_ids = HashSet::new();

        for serial_number in 1..=foundry_counter {
            let foundry_id = FoundryId::build(&alias_address, serial_number, SimpleTokenScheme::KIND);
            foundry_ids.insert(foundry_id);
        }

        foundry_ids
    }
}

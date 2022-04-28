// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions},
    Error,
};

use iota_client::bee_block::{
    address::AliasAddress,
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
    pub burn_foundries: Option<bool>,
}

impl AccountHandle {
    /// Function to destroy alias.
    pub async fn destroy_alias(
        &self,
        alias_options: AliasOptions,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] destroy_alias");

        let account = self.read().await;

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Alias(alias_output) => {
                    alias_output.alias_id().or_from_output_id(output_id) == alias_options.alias_id
                }
                _ => false,
            })
            .ok_or_else(|| Error::BurningFailed("alias not found in unspent outputs".to_string()))?;

        let (foundry_counter, custom_inputs, outputs) = match &output_data.output {
            Output::Alias(alias_output) => {
                let custom_inputs = vec![*output_id];
                let outputs = vec![Self::alias_to_basic_output(alias_output)?];
                (alias_output.foundry_counter(), custom_inputs, outputs)
            }
            _ => unreachable!("We already checked that it's an alias output"),
        };

        drop(account);

        match alias_options.burn_foundries {
            Some(burn) if !burn => {}
            _ => {
                let transfer_result = self
                    .burn_alias_foundries(alias_options.alias_id, foundry_counter, options.clone())
                    .await?;
                match transfer_result.message_id {
                    Some(message_id) => {
                        let _ = self.client.retry_until_included(&message_id, None, None).await?;
                    }
                    _ => return Err(Error::BurningFailed("Unable to burn foundries".to_string())),
                }
            }
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

        self.send(outputs, options).await
    }

    fn alias_to_basic_output(alias_output: &AliasOutput) -> crate::Result<Output> {
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

    async fn burn_alias_foundries(
        &self,
        alias_id: AliasId,
        foundry_counter: u32,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        let alias_address = AliasAddress::new(alias_id);
        let mut foundry_ids = HashSet::new();

        for serial_number in 1..=foundry_counter {
            let foundry_id = FoundryId::build(&alias_address, serial_number, SimpleTokenScheme::KIND);
            foundry_ids.insert(foundry_id);
        }

        self.burn_foundries(foundry_ids, options).await
    }
}

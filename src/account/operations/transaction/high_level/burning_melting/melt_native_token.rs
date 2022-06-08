// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_block::{
    address::AliasAddress,
    output::{
        unlock_condition::{ImmutableAliasAddressUnlockCondition, UnlockCondition},
        AliasOutputBuilder, FoundryId, FoundryOutputBuilder, Output, SimpleTokenScheme, TokenId, TokenScheme,
    },
};
use primitive_types::U256;

use crate::account::{handle::AccountHandle, operations::transaction::TransactionResult, TransactionOptions};

impl AccountHandle {
    /// Function to melt native tokens. This happens with the foundry output which minted them, by increasing it's
    /// `melted_tokens` field.
    pub async fn melt_native_token(
        &self,
        native_token: (TokenId, U256),
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSACTION] melt_native_token");
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let token_id = native_token.0;
        let melt_token_amount = native_token.1;

        let foundry_id = FoundryId::from(token_id);
        let alias_id = *foundry_id.alias_address().alias_id();

        let (existing_alias_output_data, existing_foundry_output) = self
            .find_alias_and_foundry_output_data(alias_id, foundry_id)
            .await
            .map(|(alias_data, foundry_data)| match foundry_data.output {
                Output::Foundry(foundry_output) => (alias_data, foundry_output),
                _ => unreachable!("We already checked it's a foundry output"),
            })?;

        if let Output::Alias(alias_output) = &existing_alias_output_data.output {
            // Amount can't be burned, only native tokens
            let amount = existing_alias_output_data.amount + existing_foundry_output.amount();
            // Create the new alias output with updated amount and state_index
            let alias_output = AliasOutputBuilder::from(alias_output)
                .with_alias_id(alias_id)
                .with_amount(amount)?
                .with_state_index(alias_output.state_index() + 1)
                .finish()?;

            let TokenScheme::Simple(foundry_simple_ts) = existing_foundry_output.token_scheme();
            let outputs = vec![
                Output::Alias(alias_output),
                Output::Foundry(
                    FoundryOutputBuilder::new_with_minimum_storage_deposit(
                        byte_cost_config,
                        foundry_id.serial_number(),
                        TokenScheme::Simple(SimpleTokenScheme::new(
                            *foundry_simple_ts.minted_tokens(),
                            foundry_simple_ts.melted_tokens() + melt_token_amount,
                            *foundry_simple_ts.maximum_supply(),
                        )?),
                    )?
                    .add_unlock_condition(UnlockCondition::ImmutableAliasAddress(
                        ImmutableAliasAddressUnlockCondition::new(AliasAddress::from(alias_id)),
                    ))
                    .finish()?,
                ),
            ];
            // Input selection will detect that we're melting native tokens and add the required inputs if available
            self.send(outputs, options).await
        } else {
            unreachable!("We checked if it's an alias output before")
        }
    }
}

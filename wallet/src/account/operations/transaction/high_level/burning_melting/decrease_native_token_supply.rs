// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::output::{
    AliasOutputBuilder, FoundryId, FoundryOutputBuilder, Output, SimpleTokenScheme, TokenId, TokenScheme,
};
use primitive_types::U256;

use crate::account::{handle::AccountHandle, operations::transaction::Transaction, TransactionOptions};

impl AccountHandle {
    /// Function to melt native tokens. This happens with the foundry output which minted them, by increasing it's
    /// `melted_tokens` field. This should be preferred over burning, because after burning, the foundry can never be
    /// destroyed anymore.
    pub async fn decrease_native_token_supply(
        &self,
        token_id: TokenId,
        melt_amount: U256,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        log::debug!("[TRANSACTION] decrease_native_token_supply");

        let foundry_id = FoundryId::from(token_id);
        let alias_id = *foundry_id.alias_address().alias_id();
        let token_supply = self.client.get_token_supply().await?;

        let (existing_alias_output_data, existing_foundry_output) = self
            .find_alias_and_foundry_output_data(alias_id, foundry_id)
            .await
            .map(|(alias_data, foundry_data)| match foundry_data.output {
                Output::Foundry(foundry_output) => (alias_data, foundry_output),
                _ => unreachable!("We already checked it's a foundry output"),
            })?;

        if let Output::Alias(alias_output) = &existing_alias_output_data.output {
            // Create the new alias output with updated amount and state_index
            let alias_output = AliasOutputBuilder::from(alias_output)
                .with_alias_id(alias_id)
                .with_state_index(alias_output.state_index() + 1)
                .finish_output(token_supply)?;

            let TokenScheme::Simple(token_scheme) = existing_foundry_output.token_scheme();
            let outputs = vec![
                alias_output,
                FoundryOutputBuilder::from(&existing_foundry_output)
                    .with_token_scheme(TokenScheme::Simple(SimpleTokenScheme::new(
                        token_scheme.minted_tokens(),
                        token_scheme.melted_tokens() + melt_amount,
                        token_scheme.maximum_supply(),
                    )?))
                    .finish_output(token_supply)?,
            ];
            // Input selection will detect that we're melting native tokens and add the required inputs if available
            self.send(outputs, options).await
        } else {
            unreachable!("We checked if it's an alias output before")
        }
    }
}

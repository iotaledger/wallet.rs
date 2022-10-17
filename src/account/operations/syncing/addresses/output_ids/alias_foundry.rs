// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use futures::FutureExt;
use iota_client::{
    block::{
        address::{Address, AliasAddress},
        output::{Output, OutputId},
        payload::transaction::TransactionId,
    },
    node_api::indexer::query_parameters::QueryParameter,
    Client,
};

use crate::account::handle::AccountHandle;

impl AccountHandle {
    /// Returns output ids of alias outputs and foundries owned by them
    pub(crate) async fn get_alias_and_foundry_output_ids(self, bech32_address: &str) -> crate::Result<Vec<OutputId>> {
        let client = self.client();
        let tasks = vec![
            // Get outputs where the address is in the governor address unlock condition
            async move {
                let bech32_address_ = bech32_address.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .alias_output_ids(vec![QueryParameter::Governor(bech32_address_.to_string())])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
            // Get outputs where the address is in the state controller unlock condition
            async move {
                let bech32_address_ = bech32_address.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .alias_output_ids(vec![QueryParameter::StateController(bech32_address_.to_string())])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
        ];

        // Get all results
        let mut output_ids = Vec::new();
        let results: Vec<crate::Result<Vec<OutputId>>> = futures::future::try_join_all(tasks).await?;
        for res in results {
            let found_output_ids = res?;
            output_ids.extend(found_output_ids.into_iter());
        }

        // Get alias outputs, so we can then get the foundry outputs with the alias addresses
        let alias_output_responses = self.get_outputs(output_ids.clone(), false).await?;

        let token_supply = client.get_token_supply()?;

        for alias_output_response in alias_output_responses {
            let output = Output::try_from_dto(&alias_output_response.output, token_supply)?;
            if let Output::Alias(alias_output) = output {
                let transaction_id = TransactionId::from_str(&alias_output_response.metadata.transaction_id)?;
                let output_id = OutputId::new(transaction_id, alias_output_response.metadata.output_index)?;
                let alias_address = AliasAddress::from(alias_output.alias_id().or_from_output_id(output_id));

                let foundry_output_ids = get_foundry_output_ids(client, alias_address).await?;

                output_ids.extend(foundry_output_ids.into_iter());
            }
        }

        Ok(output_ids)
    }
}

/// Returns output ids of foundries owned by the provided address
pub(crate) async fn get_foundry_output_ids(
    client: &Client,
    alias_address: AliasAddress,
) -> crate::Result<Vec<OutputId>> {
    let bech32_hrp = client.get_bech32_hrp()?;

    client
        .foundry_output_ids(vec![QueryParameter::AliasAddress(
            Address::Alias(alias_address).to_bech32(bech32_hrp.clone()),
        )])
        .await
        .map_err(From::from)
}

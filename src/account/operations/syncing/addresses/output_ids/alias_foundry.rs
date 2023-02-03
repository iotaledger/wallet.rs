// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

#[cfg(not(target_family = "wasm"))]
use futures::FutureExt;
use iota_client::{
    block::{
        address::{Address, AliasAddress},
        output::{Output, OutputId},
    },
    node_api::indexer::query_parameters::QueryParameter,
};

use crate::{
    account::{handle::AccountHandle, SyncOptions},
    task,
};

impl AccountHandle {
    /// Returns output ids of alias outputs
    pub(crate) async fn get_alias_and_foundry_output_ids(
        &self,
        bech32_address: &str,
        sync_options: SyncOptions,
    ) -> crate::Result<Vec<OutputId>> {
        log::debug!("[SYNC] get_alias_and_foundry_output_ids");
        let client = self.client();

        let mut output_ids = HashSet::new();

        #[cfg(target_family = "wasm")]
        {
            output_ids.extend(
                client
                    .alias_output_ids(vec![QueryParameter::Governor(bech32_address.to_string())])
                    .await?,
            );
            output_ids.extend(
                client
                    .alias_output_ids(vec![QueryParameter::StateController(bech32_address.to_string())])
                    .await?,
            );
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let tasks = vec![
                // Get outputs where the address is in the governor address unlock condition
                async move {
                    let bech32_address_ = bech32_address.to_string();
                    let client = client.clone();
                    task::spawn(async move {
                        client
                            .alias_output_ids(vec![QueryParameter::Governor(bech32_address_)])
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
                    task::spawn(async move {
                        client
                            .alias_output_ids(vec![QueryParameter::StateController(bech32_address_)])
                            .await
                            .map_err(From::from)
                    })
                    .await
                }
                .boxed(),
            ];
            let results: Vec<crate::Result<Vec<OutputId>>> = futures::future::try_join_all(tasks).await?;
            for res in results {
                let found_output_ids = res?;
                output_ids.extend(found_output_ids);
            }
        }

        // Get all results
        if sync_options.alias.foundry_outputs {
            let foundry_output_ids = &self.get_foundry_output_ids(&output_ids).await?;
            output_ids.extend(foundry_output_ids);
        }

        Ok(output_ids.into_iter().collect())
    }

    /// Returns output ids of foundries controlled by the provided aliases
    pub(crate) async fn get_foundry_output_ids(
        &self,
        alias_output_ids: &HashSet<OutputId>,
    ) -> crate::Result<Vec<OutputId>> {
        log::debug!("[SYNC] get_foundry_output_ids");
        // Get alias outputs, so we can then get the foundry outputs with the alias addresses
        let alias_output_responses = self.get_outputs(alias_output_ids.iter().cloned().collect()).await?;

        let bech32_hrp = self.client.get_bech32_hrp().await?;
        let token_supply = self.client.get_token_supply().await?;

        let mut tasks = vec![];

        for alias_output_response in alias_output_responses {
            let output = Output::try_from_dto(&alias_output_response.output, token_supply)?;
            if let Output::Alias(alias_output) = output {
                let output_id = alias_output_response.metadata.output_id()?;
                let alias_address = AliasAddress::from(alias_output.alias_id_non_null(&output_id));
                let alias_bech32_address = Address::Alias(alias_address).to_bech32(bech32_hrp.clone());
                let client = self.client.clone();
                tasks.push(Box::pin(task::spawn(async move {
                    client
                        .foundry_output_ids(vec![QueryParameter::AliasAddress(alias_bech32_address)])
                        .await
                        .map_err(From::from)
                })));
            }
        }

        let mut output_ids = HashSet::new();

        let results: Vec<crate::Result<Vec<OutputId>>> = futures::future::try_join_all(tasks).await?;
        for res in results {
            let foundry_output_ids = res?;
            output_ids.extend(foundry_output_ids.into_iter());
        }

        Ok(output_ids.into_iter().collect())
    }
}

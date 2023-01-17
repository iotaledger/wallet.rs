// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use iota_client::block::output::{FoundryId, Output};

use crate::account::handle::AccountHandle;

impl AccountHandle {
    pub(crate) async fn request_and_store_foundry_outputs(&self, foundry_ids: HashSet<FoundryId>) -> crate::Result<()> {
        log::debug!("[SYNC] request_and_store_foundry_outputs");

        let mut foundries = self.read().await.native_token_foundries().clone();
        let mut tasks = Vec::new();

        for foundry_id in foundry_ids {
            // Don't request known foundries again.
            if foundries.contains_key(&foundry_id) {
                continue;
            }

            let client = self.client.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    match client.foundry_output_id(foundry_id).await {
                        Ok(output_id) => Ok(Some(client.get_output(&output_id).await?)),
                        Err(iota_client::Error::NotFound(_)) => Ok(None),
                        Err(e) => Err(crate::Error::Client(e.into())),
                    }
                })
                .await
            });
        }

        let token_supply = self.client.get_token_supply().await?;
        let results = futures::future::try_join_all(tasks).await?;

        // Update account with new foundries.
        for result in results {
            if let Some(foundry_output_with_metadata) = result? {
                let output = Output::try_from_dto(&foundry_output_with_metadata.output, token_supply)?;
                if let Output::Foundry(foundry) = output {
                    foundries.insert(foundry.id(), foundry);
                }
            }
        }

        let mut account = self.write().await;
        account.native_token_foundries = foundries;

        Ok(())
    }
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use futures::FutureExt;
use iota_client::{block::output::OutputId, node_api::indexer::query_parameters::QueryParameter};

use crate::account::handle::AccountHandle;

impl AccountHandle {
    /// Returns output ids of basic outputs that have only the address unlock condition
    pub(crate) async fn get_basic_output_ids_with_address_unlock_condition_only(
        &self,
        bech32_address: String,
    ) -> iota_client::Result<Vec<OutputId>> {
        // Only request basic outputs with `AddressUnlockCondition` only
        self.client
            .basic_output_ids(vec![
                QueryParameter::Address(bech32_address),
                QueryParameter::HasExpiration(false),
                QueryParameter::HasTimelock(false),
                QueryParameter::HasStorageDepositReturn(false),
            ])
            .await
    }

    /// Returns output ids of basic outputs that have the address in the `AddressUnlockCondition`,
    /// `ExpirationUnlockCondition` or `StorageDepositReturnUnlockCondition`
    pub(crate) async fn get_basic_output_ids_with_any_unlock_condition(
        self,
        bech32_address: &str,
    ) -> crate::Result<Vec<OutputId>> {
        let client = self.client();
        let tasks = vec![
            // Get basic outputs
            async move {
                let bech32_address = bech32_address.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .basic_output_ids(vec![QueryParameter::Address(bech32_address)])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
            // Get outputs where the address is in the storage deposit return unlock condition
            async move {
                let bech32_address = bech32_address.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .basic_output_ids(vec![QueryParameter::StorageDepositReturnAddress(bech32_address)])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
            // Get outputs where the address is in an expired expiration unlock condition
            async move {
                let bech32_address = bech32_address.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .basic_output_ids(vec![
                            QueryParameter::ExpirationReturnAddress(bech32_address),
                            // Ignore outputs that aren't expired yet
                            QueryParameter::ExpiresBefore(
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("time went backwards")
                                    .as_secs() as u32,
                            ),
                        ])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
        ];
        // Get all results
        let mut output_ids = HashSet::new();
        let results: Vec<crate::Result<Vec<OutputId>>> = futures::future::try_join_all(tasks).await?;
        for res in results {
            let found_output_ids = res?;
            output_ids.extend(found_output_ids.into_iter());
        }

        Ok(output_ids.into_iter().collect())
    }
}

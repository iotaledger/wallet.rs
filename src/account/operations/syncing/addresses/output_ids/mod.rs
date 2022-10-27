// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias_foundry;
mod basic;
mod nft;

use std::{collections::HashSet, time::Instant};

use futures::FutureExt;
use iota_client::block::{address::Address, output::OutputId};

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT, handle::AccountHandle, operations::syncing::SyncOptions,
    types::address::AddressWithUnspentOutputs,
};

impl AccountHandle {
    /// Returns output ids for outputs that are directly (Ed25519 address in AddressUnlockCondition) or indirectly
    /// (alias/nft address in AddressUnlockCondition and the alias/nft output is controlled with the Ed25519 address)
    /// connected to
    pub(crate) async fn get_output_ids_for_address(
        &self,
        address: Address,
        sync_options: &SyncOptions,
    ) -> crate::Result<Vec<OutputId>> {
        let bech32_hrp = self.client.get_bech32_hrp()?;
        let bech32_address = &address.to_bech32(bech32_hrp);

        if sync_options.sync_only_most_basic_outputs {
            let output_ids = self
                .get_basic_output_ids_with_address_unlock_condition_only(bech32_address.to_string())
                .await?;
            return Ok(output_ids);
        }

        let mut tasks = vec![
            async move {
                let account_handle = self.clone();
                let bech32_address_ = bech32_address.clone();
                tokio::spawn(async move {
                    account_handle
                        .get_basic_output_ids_with_any_unlock_condition(&bech32_address_)
                        .await
                })
                .await
            }
            .boxed(),
        ];

        if sync_options.sync_aliases_and_nfts {
            // nfts
            tasks.push(
                async move {
                    let bech32_address_ = bech32_address.clone();
                    let account_handle = self.clone();
                    tokio::spawn(async move {
                        account_handle
                            .get_nft_output_ids_with_any_unlock_condition(&bech32_address_)
                            .await
                    })
                    .await
                }
                .boxed(),
            );

            // aliases and foundries
            tasks.push(
                async move {
                    let bech32_address = bech32_address.clone();
                    let account_handle = self.clone();
                    tokio::spawn(async move { account_handle.get_alias_and_foundry_output_ids(&bech32_address).await })
                        .await
                }
                .boxed(),
            );
        }

        // Get all results
        let mut output_ids = HashSet::new();
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let found_output_ids = res?;
            output_ids.extend(found_output_ids.into_iter());
        }

        Ok(output_ids.into_iter().collect())
    }

    /// Get the current output ids for provided addresses and only returns addresses that have unspent outputs and
    /// return spent outputs separated
    pub(crate) async fn get_output_ids_for_addresses(
        &self,
        options: &SyncOptions,
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    ) -> crate::Result<(Vec<AddressWithUnspentOutputs>, Vec<OutputId>)> {
        log::debug!("[SYNC] start get_output_ids_for_addresses");
        let address_output_ids_start_time = Instant::now();

        let mut addresses_with_outputs = Vec::new();
        // spent outputs or alias/nft/foundries that don't get synced anymore, because of other sync options
        let mut spent_or_not_anymore_synced_outputs = Vec::new();
        // We split the addresses into chunks so we don't get timeouts if we have thousands
        for addresses_chunk in &mut addresses_with_unspent_outputs
            .chunks(PARALLEL_REQUESTS_AMOUNT)
            .map(|x: &[AddressWithUnspentOutputs]| x.to_vec())
        {
            let mut tasks = Vec::new();
            for address in addresses_chunk {
                let account_handle = self.clone();
                let sync_options = options.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let output_ids = account_handle
                            .get_output_ids_for_address(address.address.inner, &sync_options)
                            .await?;
                        crate::Result::Ok((address, output_ids))
                    })
                    .await
                });
            }
            let results = futures::future::try_join_all(tasks).await?;
            for res in results {
                let (mut address, output_ids): (AddressWithUnspentOutputs, Vec<OutputId>) = res?;
                // only return addresses with outputs
                if !output_ids.is_empty() {
                    // outputs we had before, but now not anymore, got spent or are alias/nft/foundries that don't get
                    // synced anymore because of other sync options
                    for output_id in address.output_ids {
                        if !output_ids.contains(&output_id) {
                            spent_or_not_anymore_synced_outputs.push(output_id);
                        }
                    }
                    address.output_ids = output_ids;
                    addresses_with_outputs.push(address);
                } else {
                    // outputs we had before, but now not anymore, got spent or are alias/nft/foundries that don't get
                    // synced anymore because of other sync options
                    spent_or_not_anymore_synced_outputs.extend(address.output_ids.into_iter());
                }
            }
        }
        if options.sync_aliases_and_nfts {
            // Recursively owned outputs will be in this vec
            log::debug!(
                "[SYNC] spent or not synced outputs: {:?}",
                spent_or_not_anymore_synced_outputs
            );
        } else {
            log::debug!(
                "[SYNC] spent or not anymore synced alias/nft/foundries outputs: {:?}",
                spent_or_not_anymore_synced_outputs
            );
        }
        log::debug!(
            "[SYNC] finished get_output_ids_for_addresses in {:.2?}",
            address_output_ids_start_time.elapsed()
        );
        Ok((addresses_with_outputs, spent_or_not_anymore_synced_outputs))
    }
}

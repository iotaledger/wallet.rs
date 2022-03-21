// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT, handle::AccountHandle, operations::syncing::SyncOptions,
    types::address::AddressWithBalance, OutputData,
};
#[cfg(feature = "events")]
use crate::events::types::WalletEvent;

use iota_client::{bee_message::output::OutputId, node_api::indexer_api::query_parameters::QueryParameter};

use std::time::Instant;
impl AccountHandle {
    /// Get the balance and return only addresses with a positive balance
    pub(crate) async fn get_addresses_to_sync(&self, options: &SyncOptions) -> crate::Result<Vec<AddressWithBalance>> {
        log::debug!("[SYNC] get_addresses_to_sync");
        let balance_sync_start_time = Instant::now();

        let mut addresses_before_syncing = self.list_addresses().await?;
        // Filter addresses when address_start_index is not 0 so we skip these addresses
        // If we force syncing, we want to sync all addresses
        if options.address_start_index != 0 && !options.force_syncing {
            addresses_before_syncing = addresses_before_syncing
                .into_iter()
                .filter(|a| a.key_index >= options.address_start_index)
                .collect();
        }

        let addresses_with_balance = self.list_addresses_with_balance().await?;
        let mut addresses_with_old_output_ids = Vec::new();
        for address in addresses_before_syncing {
            let mut output_ids = Vec::new();
            if let Some(address_with_balance) = addresses_with_balance.iter().find(|a| a.address == address.address) {
                output_ids = address_with_balance.output_ids.to_vec();
            }
            addresses_with_old_output_ids.push(AddressWithBalance {
                address: address.address,
                key_index: address.key_index,
                internal: address.internal,
                amount: 0,
                output_ids,
            })
        }

        Ok(addresses_with_old_output_ids)
    }

    /// Get the current output ids for provided addresses and only returns addresses that have outputs now and return
    /// spent outputs separated
    pub(crate) async fn get_address_output_ids(
        &self,
        options: &SyncOptions,
        addresses_with_balance: Vec<AddressWithBalance>,
    ) -> crate::Result<(Vec<AddressWithBalance>, Vec<OutputId>)> {
        log::debug!("[SYNC] start get_address_output_ids");
        let address_output_ids_start_time = Instant::now();
        let account = self.read().await;

        #[cfg(feature = "events")]
        let (account_index, consolidation_threshold) =
            (account.index, account.account_options.output_consolidation_threshold);
        drop(account);

        let mut addresses_with_outputs = Vec::new();
        // spent outputs or alias/nft/foundries that don't get synced anymore, because of other sync options
        let mut spent_or_not_anymore_synced_outputs = Vec::new();
        // We split the addresses into chunks so we don't get timeouts if we have thousands
        for addresses_chunk in &mut addresses_with_balance
            .chunks(PARALLEL_REQUESTS_AMOUNT)
            .map(|x: &[AddressWithBalance]| x.to_vec())
        {
            let mut tasks = Vec::new();
            for address in addresses_chunk {
                let client = self.client.clone();
                let sync_options = options.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let client = client;
                        // Get basic outputs
                        let mut output_ids = client
                            .output_ids(vec![QueryParameter::Address(address.address.to_bech32())])
                            .await?;

                        if sync_options.sync_aliases_and_nfts {
                            // Get nft outputs
                            output_ids.extend(
                                client
                                    .nfts_output_ids(vec![
                                        QueryParameter::Address(address.address.to_bech32()),
                                        // todo: handle the following unlock conditions
                                        QueryParameter::HasExpirationCondition(false),
                                        QueryParameter::HasTimelockCondition(false),
                                        QueryParameter::HasStorageDepositReturnCondition(false),
                                    ])
                                    .await?
                                    .into_iter(),
                            );
                            // Get alias outputs
                            output_ids.extend(
                                client
                                    .aliases_output_ids(vec![
                                        QueryParameter::StateController(address.address.to_bech32()),
                                        QueryParameter::Governor(address.address.to_bech32()),
                                    ])
                                    .await?
                                    .into_iter(),
                            );
                            // todo for alias check if there are foundries (here or later after we fetched the outputs?)
                            // Maybe the cleanest way is to do it only when syncing again, when we get the addresses for
                            // syncing we could add alias addresses
                        }
                        crate::Result::Ok((address, output_ids))
                    })
                    .await
                });
            }
            let results = futures::future::try_join_all(tasks).await?;
            for res in results {
                let (mut address, output_ids): (AddressWithBalance, Vec<OutputId>) = res?;
                #[cfg(feature = "events")]
                if output_ids.len() > consolidation_threshold {
                    self.event_emitter
                        .lock()
                        .await
                        .emit(account_index, WalletEvent::ConsolidationRequired);
                }
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
            log::debug!("[SYNC] spent outputs: {:?}", spent_or_not_anymore_synced_outputs);
        } else {
            log::debug!(
                "[SYNC] spent or not anymore synced alias/nft/foundries outputs: {:?}",
                spent_or_not_anymore_synced_outputs
            );
        }
        log::debug!(
            "[SYNC] finished get_address_output_ids in {:.2?}",
            address_output_ids_start_time.elapsed()
        );
        Ok((addresses_with_outputs, spent_or_not_anymore_synced_outputs))
    }

    /// Get outputs from addresses
    pub(crate) async fn get_addresses_outputs(
        &self,
        addresses_with_balance: Vec<AddressWithBalance>,
    ) -> crate::Result<(Vec<AddressWithBalance>, Vec<OutputData>)> {
        log::debug!("[SYNC] start get_addresses_outputs");
        let address_outputs_start_time = Instant::now();

        let mut addresses_with_outputs = Vec::new();
        let mut outputs_data = Vec::new();

        // We split the addresses into chunks so we don't get timeouts if we have thousands
        for addresses_chunk in &mut addresses_with_balance
            .chunks(PARALLEL_REQUESTS_AMOUNT)
            .map(|x: &[AddressWithBalance]| x.to_vec())
        {
            let mut tasks = Vec::new();
            for mut address in addresses_chunk {
                let account_handle = self.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let (output_responses, already_known_balance) =
                            account_handle.get_outputs(address.output_ids.clone(), false).await?;
                        let outputs = account_handle
                            .output_response_to_output_data(output_responses, &address)
                            .await?;
                        // Add balance from new outputs together with balance from already known outputs
                        address.amount =
                            outputs.iter().map(|output| output.amount).sum::<u64>() + already_known_balance;
                        crate::Result::Ok((address, outputs))
                    })
                    .await
                });
            }
            let results = futures::future::try_join_all(tasks).await?;
            for res in results {
                let (address, outputs): (AddressWithBalance, Vec<OutputData>) = res?;
                addresses_with_outputs.push(address);
                outputs_data.extend(outputs.into_iter());
            }
        }
        log::debug!(
            "[SYNC] finished get_address_output_ids in {:.2?}",
            address_outputs_start_time.elapsed()
        );
        Ok((addresses_with_outputs, outputs_data))
    }
}

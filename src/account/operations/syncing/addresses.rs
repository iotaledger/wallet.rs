// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashSet, str::FromStr, time::Instant};

use futures::FutureExt;
use iota_client::{
    block::{
        address::{Address, AliasAddress, NftAddress},
        output::{Output, OutputId},
        payload::transaction::TransactionId,
    },
    node_api::indexer::query_parameters::QueryParameter,
    Client,
};

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT,
    handle::AccountHandle,
    operations::syncing::{OutputResponse, SyncOptions},
    types::address::AddressWithUnspentOutputs,
    OutputData,
};

impl AccountHandle {
    /// Get the addresses that should be synced with the current known unspent output ids
    /// Also adds alias and nft addresses from unspent alias or nft outputs that have no Timelock, Expiration or
    /// StorageDepositReturn [`UnlockCondition`]
    pub(crate) async fn get_addresses_to_sync(
        &self,
        options: &SyncOptions,
    ) -> crate::Result<Vec<AddressWithUnspentOutputs>> {
        log::debug!("[SYNC] get_addresses_to_sync");

        let mut addresses_before_syncing = self.list_addresses().await?;

        // If custom addresses are provided check if they are in the account and only use them
        if !options.addresses.is_empty() {
            let mut specific_addresses_to_sync = HashSet::new();
            for bech32_address in &options.addresses {
                let (_bech32_hrp, address) = Address::try_from_bech32(bech32_address)?;
                match addresses_before_syncing.iter().find(|a| a.address.inner == address) {
                    Some(address) => {
                        specific_addresses_to_sync.insert(address.clone());
                    }
                    None => return Err(crate::Error::AddressNotFoundInAccount(bech32_address.to_string())),
                }
            }
            addresses_before_syncing = specific_addresses_to_sync.into_iter().collect();
        }

        // Filter addresses when address_start_index is not 0 so we skip these addresses
        // If we force syncing, we want to sync all addresses
        if options.addresses.is_empty() && options.address_start_index != 0 && !options.force_syncing {
            addresses_before_syncing = addresses_before_syncing
                .into_iter()
                .filter(|a| a.key_index >= options.address_start_index)
                .collect();
        }

        // Check if selected addresses contains addresses with balance so we can correctly update them
        let addresses_with_unspent_outputs = self.list_addresses_with_unspent_outputs().await?;
        let mut addresses_with_old_output_ids = Vec::new();
        for address in addresses_before_syncing {
            let mut output_ids = Vec::new();
            // Add currently known unspent output ids, so we can later compare them with the new output ids and see if
            // one got spent (is missing in the new returned output ids)
            if let Some(address_with_unpsnet_outputs) = addresses_with_unspent_outputs
                .iter()
                .find(|a| a.address == address.address)
            {
                output_ids = address_with_unpsnet_outputs.output_ids.to_vec();
            }
            addresses_with_old_output_ids.push(AddressWithUnspentOutputs {
                address: address.address,
                key_index: address.key_index,
                internal: address.internal,
                output_ids,
            })
        }

        Ok(addresses_with_old_output_ids)
    }

    /// Get the current output ids for provided addresses and only returns addresses that have unspent outputs and
    /// return spent outputs separated
    pub(crate) async fn get_address_output_ids(
        &self,
        options: &SyncOptions,
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    ) -> crate::Result<(Vec<AddressWithUnspentOutputs>, Vec<OutputId>)> {
        log::debug!("[SYNC] start get_address_output_ids");
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
                let client = self.client.clone();
                let account_handle = self.clone();
                let sync_options = options.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let client = client;
                        account_handle
                            .request_address_output_ids(&client, address, &sync_options)
                            .await
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
        addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    ) -> crate::Result<(Vec<AddressWithUnspentOutputs>, Vec<OutputData>)> {
        log::debug!("[SYNC] start get_addresses_outputs");
        let address_outputs_start_time = Instant::now();

        let mut addresses_with_outputs = Vec::new();
        let mut outputs_data = Vec::new();

        // We split the addresses into chunks so we don't get timeouts if we have thousands
        for addresses_chunk in &mut addresses_with_unspent_outputs
            .chunks(PARALLEL_REQUESTS_AMOUNT)
            .map(|x: &[AddressWithUnspentOutputs]| x.to_vec())
        {
            let mut tasks = Vec::new();
            for address in addresses_chunk {
                let account_handle = self.clone();
                tasks.push(async move {
                    tokio::spawn(async move {
                        let (output_responses, _loaded_output_responses) =
                            account_handle.get_outputs(address.output_ids.clone(), false).await?;

                        let outputs = account_handle
                            .output_response_to_output_data(output_responses, &address)
                            .await?;
                        crate::Result::Ok((address, outputs))
                    })
                    .await
                });
            }
            let results = futures::future::try_join_all(tasks).await?;
            for res in results {
                let (address, outputs): (AddressWithUnspentOutputs, Vec<OutputData>) = res?;
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

    /// Returns output ids for outputs that are directly (Ed25519 address in AddressUnlockCondition) or indirectly
    /// (alias/nft address in AddressUnlockCondition and the alias/nft output is controlled with the Ed25519 address)
    /// connected to
    pub(crate) async fn request_address_output_ids(
        &self,
        client: &Client,
        address: AddressWithUnspentOutputs,
        sync_options: &SyncOptions,
    ) -> crate::Result<(AddressWithUnspentOutputs, Vec<OutputId>)> {
        let bech32_address_ = &address.address.to_bech32();

        // Only sync outputs with basic unlock conditions and only `AddressUnlockCondition`
        if sync_options.sync_only_most_basic_outputs {
            let output_ids = client
                .basic_output_ids(vec![
                    QueryParameter::Address(bech32_address_.to_string()),
                    QueryParameter::HasExpiration(false),
                    QueryParameter::HasTimelock(false),
                    QueryParameter::HasStorageDepositReturn(false),
                ])
                .await?;
            return Ok((address, output_ids));
        }

        let mut tasks = vec![
            // Get basic outputs
            async move {
                let bech32_address = bech32_address_.to_string();
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
                let bech32_address = bech32_address_.to_string();
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
            // Get outputs where the address is in the expiration unlock condition
            async move {
                let bech32_address = bech32_address_.to_string();
                let client = client.clone();
                tokio::spawn(async move {
                    client
                        .basic_output_ids(vec![QueryParameter::ExpirationReturnAddress(bech32_address)])
                        .await
                        .map_err(From::from)
                })
                .await
            }
            .boxed(),
        ];

        if sync_options.sync_aliases_and_nfts {
            // nfts
            let account_handle = self.clone();
            let bech32_hrp = address.address.bech32_hrp.clone();
            tasks.push(
                async move {
                    let bech32_address = bech32_address_.to_string();
                    let client = client.clone();
                    tokio::spawn(async move {
                        let mut output_ids = Vec::new();
                        // Get nft outputs
                        let nft_output_ids = client
                            .nft_output_ids(vec![QueryParameter::Address(bech32_address.to_string())])
                            .await?;
                        output_ids.extend(nft_output_ids.clone().into_iter());

                        // Get outputs where the address is in the storage deposit return unlock condition
                        let nft_output_ids = client
                            .nft_output_ids(vec![QueryParameter::StorageDepositReturnAddress(
                                bech32_address.to_string(),
                            )])
                            .await?;
                        output_ids.extend(nft_output_ids.clone().into_iter());

                        // Get outputs where the address is in the expiration unlock condition
                        let nft_output_ids = client
                            .nft_output_ids(vec![QueryParameter::ExpirationReturnAddress(
                                bech32_address.to_string(),
                            )])
                            .await?;
                        output_ids.extend(nft_output_ids.clone().into_iter());

                        // get basic outputs that can be controlled by an nft output
                        let (mut nft_output_responses, loaded_output_responses) =
                            account_handle.get_outputs(nft_output_ids, false).await?;
                        nft_output_responses.extend(loaded_output_responses.into_iter());
                        let nft_basic_output_ids =
                            get_basic_outputs_for_nft_outputs(&client, nft_output_responses, bech32_hrp).await?;
                        output_ids.extend(nft_basic_output_ids.into_iter());
                        Ok(output_ids)
                    })
                    .await
                }
                .boxed(),
            );

            // aliases
            let account_handle = self.clone();
            let bech32_hrp = address.address.bech32_hrp.clone();
            tasks.push(
                async move {
                    let bech32_address = bech32_address_.to_string();
                    let client = client.clone();
                    tokio::spawn(async move {
                        let mut output_ids = Vec::new();
                        // Get alias outputs
                        let alias_output_ids = client
                            .alias_output_ids(vec![
                                QueryParameter::StateController(bech32_address.to_string()),
                                QueryParameter::Governor(bech32_address.to_string()),
                            ])
                            .await?;
                        output_ids.extend(alias_output_ids.clone().into_iter());

                        // get possible foundries and basic outputs that can be controlled by an alias outputs
                        let (mut alias_output_responses, loaded_output_responses) =
                            account_handle.get_outputs(alias_output_ids, false).await?;
                        alias_output_responses.extend(loaded_output_responses.into_iter());
                        let alias_foundry_and_basic_output_ids = get_foundry_and_basic_outputs_for_alias_outputs(
                            &client,
                            alias_output_responses,
                            bech32_hrp,
                        )
                        .await?;
                        output_ids.extend(alias_foundry_and_basic_output_ids.into_iter());
                        crate::Result::Ok(output_ids)
                    })
                    .await
                }
                .boxed(),
            );
        }

        // Get all results
        let mut output_ids = Vec::new();
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let found_output_ids = res?;
            output_ids.extend(found_output_ids.into_iter());
        }

        // Dedup since the same output id could be returned from different queries.
        output_ids.sort();
        output_ids.dedup();

        Ok((address, output_ids))
    }
}

// Get basic outputs that have the [`NftAddress`] from nft outputs in their [`AddressUnlockCondition`]
async fn get_basic_outputs_for_nft_outputs(
    client: &Client,
    nft_output_responses: Vec<OutputResponse>,
    bech32_hrp: String,
) -> crate::Result<Vec<OutputId>> {
    let mut nft_basic_output_ids = Vec::new();
    for nft_output_response in nft_output_responses {
        let output = Output::try_from(&nft_output_response.output)?;
        if let Output::Nft(nft_output) = output {
            let transaction_id = TransactionId::from_str(&nft_output_response.metadata.transaction_id)?;
            let output_id = OutputId::new(transaction_id, nft_output_response.metadata.output_index)?;
            let nft_address = Address::Nft(NftAddress::new(nft_output.nft_id().or_from_output_id(output_id)));
            nft_basic_output_ids.extend(
                client
                    .basic_output_ids(vec![QueryParameter::Address(nft_address.to_bech32(bech32_hrp.clone()))])
                    .await?
                    .into_iter(),
            );
        }
    }
    Ok(nft_basic_output_ids)
}

// Get basic outputs that have the [`AliasAddress`] from alias outputs in their [`AddressUnlockCondition`]
async fn get_foundry_and_basic_outputs_for_alias_outputs(
    client: &Client,
    alias_output_responses: Vec<OutputResponse>,
    bech32_hrp: String,
) -> crate::Result<Vec<OutputId>> {
    let mut foundry_output_ids = Vec::new();
    let mut alias_basic_output_ids = Vec::new();
    for alias_output_response in alias_output_responses {
        let output = Output::try_from(&alias_output_response.output)?;
        if let Output::Alias(alias_output) = output {
            let transaction_id = TransactionId::from_str(&alias_output_response.metadata.transaction_id)?;
            let output_id = OutputId::new(transaction_id, alias_output_response.metadata.output_index)?;
            let alias_address =
                Address::Alias(AliasAddress::from(alias_output.alias_id().or_from_output_id(output_id)));
            foundry_output_ids.extend(
                client
                    .foundry_output_ids(vec![QueryParameter::AliasAddress(
                        alias_address.to_bech32(bech32_hrp.clone()),
                    )])
                    .await?
                    .into_iter(),
            );
            alias_basic_output_ids.extend(
                client
                    .basic_output_ids(vec![QueryParameter::Address(
                        alias_address.to_bech32(bech32_hrp.clone()),
                    )])
                    .await?
                    .into_iter(),
            );
        }
    }
    // Add output ids together
    foundry_output_ids.extend(alias_basic_output_ids.into_iter());
    Ok(foundry_output_ids)
}

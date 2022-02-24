// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT, handle::AccountHandle, operations::syncing::SyncOptions,
    types::address::AddressWithBalance,
};
#[cfg(feature = "events")]
use crate::events::types::WalletEvent;

use iota_client::{bee_message::output::OutputId, node_api::indexer_api::query_parameters::QueryParameter};

use std::time::Instant;

/// Get the balance and return only addresses with a positive balance
pub(crate) async fn get_addresses_to_sync(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<AddressWithBalance>> {
    log::debug!("[SYNC] get_addresses_to_sync");
    let balance_sync_start_time = Instant::now();

    let mut addresses_before_syncing = account_handle.list_addresses().await?;
    // Filter addresses when address_start_index is not 0 so we skip these addresses
    if options.address_start_index != 0 {
        addresses_before_syncing = addresses_before_syncing
            .into_iter()
            .filter(|a| a.key_index >= options.address_start_index)
            .collect();
    }

    Ok(addresses_before_syncing
        .into_iter()
        .map(|address| AddressWithBalance {
            address: address.address,
            key_index: address.key_index,
            internal: address.internal,
            amount: 0,
            output_ids: Vec::new(),
        })
        .collect())
}

/// Get the current output ids for provided addresses and only returns addresses that have outputs
pub(crate) async fn get_address_output_ids(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_balance: Vec<AddressWithBalance>,
) -> crate::Result<Vec<AddressWithBalance>> {
    log::debug!("[SYNC] start get_address_output_ids");
    let address_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    #[cfg(feature = "events")]
    let (account_index, consolidation_threshold) =
        (account.index, account.account_options.output_consolidation_threshold);
    drop(account);

    let mut addresses_with_outputs = Vec::new();
    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in &mut addresses_with_balance
        .chunks(PARALLEL_REQUESTS_AMOUNT)
        .map(|x: &[AddressWithBalance]| x.to_vec())
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let client = account_handle.client.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client;
                    let output_ids = client
                        .output_ids(vec![QueryParameter::Address(address.address.to_bech32())])
                        .await?;

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
                account_handle
                    .event_emitter
                    .lock()
                    .await
                    .emit(account_index, WalletEvent::ConsolidationRequired);
            }
            // only return addresses with outputs
            if !output_ids.is_empty() {
                address.output_ids = output_ids;
                addresses_with_outputs.push(address);
            }
        }
    }
    log::debug!(
        "[SYNC] finished get_address_output_ids in {:.2?}",
        address_outputs_sync_start_time.elapsed()
    );
    Ok(addresses_with_outputs)
}

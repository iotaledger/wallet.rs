// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT,
    handle::AccountHandle,
    operations::syncing::SyncOptions,
    types::address::{AccountAddress, AddressWithBalance},
};
#[cfg(feature = "events")]
use crate::events::types::WalletEvent;

use iota_client::{bee_message::output::OutputId, bee_rest_api::types::responses::OutputsAddressResponse};

use std::{str::FromStr, time::Instant};

/// Get the balance and return only addresses with a positive balance
pub(crate) async fn get_addresses_with_balance(
    account_handle: &AccountHandle,
    options: &SyncOptions,
) -> crate::Result<Vec<AddressWithBalance>> {
    log::debug!("[SYNC] start get_addresses_with_balance");
    let balance_sync_start_time = Instant::now();

    let mut address_before_syncing = account_handle.list_addresses().await?;
    // Filter addresses when address_start_index is not 0 so we skip these addresses
    if options.address_start_index != 0 {
        address_before_syncing = address_before_syncing
            .into_iter()
            .filter(|a| a.key_index >= options.address_start_index)
            .collect();
    }

    let account = account_handle.read().await;
    drop(account);

    log::debug!("[SYNC] sync balance for {} addresses", address_before_syncing.len());
    let client = crate::client::get_client().await?;
    let mut addresses_with_balance = Vec::new();
    for addresses_chunk in address_before_syncing
        .chunks(PARALLEL_REQUESTS_AMOUNT)
        .map(|x: &[AccountAddress]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let client = client.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client;
                    let balance_response = client.get_address().balance(&address.address().to_bech32()).await?;
                    if balance_response.balance != 0 {
                        log::debug!(
                            "[SYNC] found {}i on {}",
                            balance_response.balance,
                            address.address().to_bech32()
                        );
                    }

                    crate::Result::Ok(AddressWithBalance {
                        address: address.address,
                        key_index: address.key_index,
                        internal: address.internal,
                        balance: balance_response.balance,
                        output_ids: Vec::new(),
                    })
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let address = res?;
            // only return addresses with balance or if we discover an account so we don't pass empty addresses around
            // which only slows the process down
            if address.balance != 0 || options.sync_all_addresses {
                addresses_with_balance.push(address);
            }
        }
    }
    log::debug!(
        "[SYNC] finished get_addresses_with_balance in {:.2?}",
        balance_sync_start_time.elapsed()
    );
    Ok(addresses_with_balance)
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_address_output_ids(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    addresses_with_balance: Vec<AddressWithBalance>,
) -> crate::Result<(Vec<OutputId>, Vec<AddressWithBalance>)> {
    log::debug!("[SYNC] start get_address_output_ids");
    let address_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client = crate::client::get_client().await?;
    #[cfg(feature = "events")]
    let (account_index, consolidation_threshold) =
        (account.index, account.account_options.output_consolidation_threshold);
    drop(account);

    let mut found_outputs = Vec::new();
    let mut addresses_with_outputs = Vec::new();
    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in &mut addresses_with_balance
        .chunks(PARALLEL_REQUESTS_AMOUNT)
        .map(|x: &[AddressWithBalance]| x.to_vec())
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let client = client.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client;
                    let outputs_response = client
                        .get_address()
                        .outputs_response(&address.address().to_bech32(), Default::default())
                        .await?;
                    crate::Result::Ok((address, outputs_response))
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let (mut address, outputs_response): (AddressWithBalance, OutputsAddressResponse) = res?;
            if !outputs_response.output_ids.is_empty() || options.sync_all_addresses {
                let mut address_outputs = Vec::new();
                for output_id in &outputs_response.output_ids {
                    found_outputs.push(OutputId::from_str(output_id)?);
                    address_outputs.push(OutputId::from_str(output_id)?);
                }
                address.output_ids = address_outputs;
                addresses_with_outputs.push(address);
                #[cfg(feature = "events")]
                if outputs_response.output_ids.len() > consolidation_threshold {
                    account_handle
                        .event_emitter
                        .lock()
                        .await
                        .emit(account_index, WalletEvent::ConsolidationRequired);
                }
            }
        }
    }
    log::debug!(
        "[SYNC] finished get_address_output_ids in {:.2?}",
        address_outputs_sync_start_time.elapsed()
    );
    // addresses with current outputs, historic outputs are ignored
    Ok((found_outputs, addresses_with_outputs))
}

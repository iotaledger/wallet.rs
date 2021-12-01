// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        operations::{address_generation::AddressGenerationOptions, syncing::SyncOptions},
        types::AccountBalance,
    },
    signing::GenerateAddressMetadata,
};

use std::cmp;

/// Search addresses with funds
/// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
/// has balance, the counter is reset
/// Addresses that got crated during this operation and have a higher key_index than the latest one with balance,
/// will be removed again, to keep the account size smaller
pub(crate) async fn search_addresses_with_funds(
    account_handle: &AccountHandle,
    address_gap_limit: usize,
) -> crate::Result<AccountBalance> {
    log::debug!("[search_addresses_with_funds]");
    let client = crate::client::get_client().await?;
    let bech32_hrp = client.get_bech32_hrp().await?;
    let network = match bech32_hrp.as_str() {
        "iota" => crate::signing::Network::Mainnet,
        _ => crate::signing::Network::Testnet,
    };

    // store the length so we can remove addresses with higher indexes later if they don't have balance
    let (highest_public_address_index, highest_internal_address_index) = {
        let account = account_handle.read().await;
        let highest_public_address_index = match account.public_addresses.last() {
            Some(a) => a.key_index,
            None => 0,
        };
        let highest_internal_address_index = match account.internal_addresses.last() {
            Some(a) => a.key_index,
            None => 0,
        };
        (highest_public_address_index, highest_internal_address_index)
    };

    let mut latest_balance = 0;
    loop {
        // generate public and internal addresses
        let addresses = account_handle
            .generate_addresses(
                address_gap_limit,
                Some(AddressGenerationOptions {
                    internal: false,
                    metadata: GenerateAddressMetadata {
                        network: network.clone(),
                        syncing: true,
                    },
                }),
            )
            .await?;
        account_handle
            .generate_addresses(
                address_gap_limit,
                Some(AddressGenerationOptions {
                    internal: true,
                    metadata: GenerateAddressMetadata {
                        network: network.clone(),
                        syncing: true,
                    },
                }),
            )
            .await?;

        let balance = account_handle
            .sync(Some(SyncOptions {
                force_syncing: true,
                // skip previous addresses
                address_start_index: match addresses.first() {
                    Some(address) => address.key_index,
                    None => 0,
                },
                ..Default::default()
            }))
            .await?;

        // break if we didn't find more balance with the new addresses
        if balance.total <= latest_balance {
            break;
        }
        latest_balance = balance.total;
    }

    clean_account_after_recovery(
        account_handle,
        highest_public_address_index,
        highest_internal_address_index,
    )
    .await;

    account_handle
        .sync(Some(SyncOptions {
            sync_all_addresses: true,
            force_syncing: true,
            ..Default::default()
        }))
        .await
}

/// During search_addresses_with_funds we created new addresses that don't have funds, so we remove them again
/// addresses_len was before we generated new addresses in search_addresses_with_funds
async fn clean_account_after_recovery(
    account_handle: &AccountHandle,
    old_highest_public_address_index: usize,
    old_highest_internal_address_index: usize,
) -> AccountHandle {
    let mut account = account_handle.write().await;
    let addresses_with_balance = account.addresses_with_balance().iter().filter(|a| a.balance != 0);
    let highest_public_index_with_balance = addresses_with_balance
        .clone()
        .filter(|a| !a.internal)
        .map(|a| a.key_index)
        .max()
        // We want to have at least one address
        .unwrap_or(0);
    let highest_internal_index_with_balance = addresses_with_balance
        .filter(|a| a.internal)
        .map(|a| a.key_index)
        .max()
        .unwrap_or(0);

    // The new highest index should be either the old one before we searched for funds or if we found addresses with
    // funds the highest index from an address with balance
    let new_latest_public_index = cmp::max(highest_public_index_with_balance, old_highest_public_address_index);
    account.public_addresses = account
        .public_addresses
        .clone()
        .into_iter()
        .filter(|a| a.key_index <= new_latest_public_index)
        .collect();
    let new_latest_internal_index = cmp::max(highest_internal_index_with_balance, old_highest_internal_address_index);
    account.internal_addresses = account
        .internal_addresses
        .clone()
        .into_iter()
        .filter(|a| a.key_index <= new_latest_internal_index)
        .collect();
    account_handle.clone()
}

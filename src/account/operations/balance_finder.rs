// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::cmp;

use iota_client::secret::GenerateAddressMetadata;

use crate::account::{
    handle::AccountHandle,
    operations::{address_generation::AddressGenerationOptions, syncing::SyncOptions},
    types::AccountBalance,
};

impl AccountHandle {
    /// Search addresses with funds
    /// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
    /// has balance, the counter is reset
    /// Addresses that got crated during this operation and have a higher key_index than the latest one with balance,
    /// will be removed again, to keep the account size smaller
    pub(crate) async fn search_addresses_with_funds(
        self: &AccountHandle,
        address_gap_limit: u32,
    ) -> crate::Result<AccountBalance> {
        log::debug!("[search_addresses_with_funds]");

        // store the length so we can remove addresses with higher indexes later if they don't have balance
        let (highest_public_address_index, highest_internal_address_index) = {
            let account = self.read().await;
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

        let mut latest_balance: Option<AccountBalance> = None;
        loop {
            // generate public and internal addresses
            let addresses = self
                .generate_addresses(
                    address_gap_limit,
                    Some(AddressGenerationOptions {
                        internal: false,
                        metadata: GenerateAddressMetadata { syncing: true },
                    }),
                )
                .await?;
            self.generate_addresses(
                address_gap_limit,
                Some(AddressGenerationOptions {
                    internal: true,
                    metadata: GenerateAddressMetadata { syncing: true },
                }),
            )
            .await?;

            let balance = self
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
            if balance.total <= latest_balance.as_ref().map_or(&AccountBalance::default(), |v| v).total {
                latest_balance.replace(balance);
                break;
            }
            latest_balance.replace(balance);
        }

        self.clean_account_after_recovery(highest_public_address_index, highest_internal_address_index)
            .await;

        Ok(latest_balance.unwrap_or_default())
    }

    /// During search_addresses_with_funds we created new addresses that don't have funds, so we remove them again
    /// addresses_len was before we generated new addresses in search_addresses_with_funds
    async fn clean_account_after_recovery(
        &self,
        old_highest_public_address_index: u32,
        old_highest_internal_address_index: u32,
    ) -> AccountHandle {
        let mut account = self.write().await;
        let addresses_with_unspent_outputs = account
            .addresses_with_unspent_outputs()
            .iter()
            .filter(|a| a.amount != 0);
        let highest_public_index_with_balance = addresses_with_unspent_outputs
            .clone()
            .filter(|a| !a.internal)
            .map(|a| a.key_index)
            .max()
            // We want to have at least one address
            .unwrap_or(0);
        let highest_internal_index_with_balance = addresses_with_unspent_outputs
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
        let new_latest_internal_index =
            cmp::max(highest_internal_index_with_balance, old_highest_internal_address_index);
        // For internal addresses we don't leave an empty address, that's only required for the public address
        account.internal_addresses = if new_latest_internal_index == 0 {
            Vec::new()
        } else {
            account
                .internal_addresses
                .clone()
                .into_iter()
                .filter(|a| a.key_index <= new_latest_internal_index)
                .collect()
        };
        self.clone()
    }
}

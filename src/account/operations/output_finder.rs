// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::cmp;

use iota_client::secret::GenerateAddressOptions;

use crate::account::{
    handle::AccountHandle,
    operations::{address_generation::AddressGenerationOptions, syncing::SyncOptions},
    types::AddressWithUnspentOutputs,
};

impl AccountHandle {
    /// Search addresses with unspent outputs
    /// `address_gap_limit`: The number of addresses to search for, after the last address with unspent outputs
    /// Addresses that got crated during this operation and have a higher key_index than the latest one with outputs,
    /// will be removed again, to keep the account size smaller
    pub(crate) async fn search_addresses_with_outputs(
        self: &AccountHandle,
        mut address_gap_limit: u32,
        mut sync_options: Option<SyncOptions>,
    ) -> crate::Result<usize> {
        log::debug!("[search_addresses_with_outputs]");

        // store the current index, so we can remove new addresses with higher indexes later again, if they don't have
        // outputs
        let (highest_public_address_index, highest_internal_address_index) = {
            let account = self.read().await;
            (
                account
                    .public_addresses
                    .last()
                    .map(|a| a.key_index)
                    .expect("account needs to have a public address"),
                account.internal_addresses.last().map(|a| a.key_index),
            )
        };

        // Generate addresses below the start indexes
        if let Some(sync_options) = &sync_options {
            // public addresses
            if sync_options.address_start_index != 0 {
                let mut address_amount_to_generate =
                    sync_options.address_start_index.abs_diff(highest_public_address_index);
                // -1 if it's larger than 0, to get the correct amount, because the address with the actual start index
                // gets generated later
                if address_amount_to_generate > 0 {
                    address_amount_to_generate -= 1;
                }
                log::debug!(
                    "[search_addresses_with_outputs] generate {address_amount_to_generate} public addresses below the start index"
                );
                self.generate_addresses(
                    address_amount_to_generate,
                    Some(AddressGenerationOptions {
                        internal: false,
                        metadata: GenerateAddressOptions {
                            ledger_nano_prompt: true,
                        },
                    }),
                )
                .await?;
            }
            // internal addresses
            if sync_options.address_start_index_internal != 0 {
                let mut address_amount_to_generate = sync_options
                    .address_start_index_internal
                    .abs_diff(highest_internal_address_index.unwrap_or(0));
                // -1 if it's larger than 0, to get the correct amount, because the address with the actual start index
                // gets generated later
                if address_amount_to_generate > 0 && highest_internal_address_index.is_some() {
                    address_amount_to_generate -= 1;
                }
                log::debug!(
                    "[search_addresses_with_outputs] generate {address_amount_to_generate} internal addresses below the start index"
                );
                self.generate_addresses(
                    address_amount_to_generate,
                    Some(AddressGenerationOptions {
                        internal: true,
                        metadata: GenerateAddressOptions {
                            ledger_nano_prompt: true,
                        },
                    }),
                )
                .await?;
            }
        }

        let mut address_gap_limit_internal = address_gap_limit;

        let mut latest_outputs_count = 0;
        loop {
            // Also needs to be in the loop so it gets updated every round for internal use without modifying the values
            // outside
            let (highest_public_address_index, highest_internal_address_index) = {
                let account = self.read().await;
                (
                    account
                        .public_addresses
                        .last()
                        .map(|a| a.key_index)
                        .expect("account needs to have a public address"),
                    account.internal_addresses.last().map(|a| a.key_index),
                )
            };
            log::debug!(
                "[search_addresses_with_outputs] address_gap_limit: {address_gap_limit}, address_gap_limit_internal: {address_gap_limit_internal}"
            );
            // generate public and internal addresses
            let addresses = self
                .generate_addresses(
                    address_gap_limit,
                    Some(AddressGenerationOptions {
                        internal: false,
                        metadata: GenerateAddressOptions {
                            ledger_nano_prompt: true,
                        },
                    }),
                )
                .await?;
            let internal_addresses = self
                .generate_addresses(
                    address_gap_limit_internal,
                    Some(AddressGenerationOptions {
                        internal: true,
                        metadata: GenerateAddressOptions {
                            ledger_nano_prompt: true,
                        },
                    }),
                )
                .await?;

            let address_start_index = addresses
                .first()
                .map(|a| {
                    // If the index is 1, then we only have the single address before we got during account creation
                    // To also sync that, we set the index to 0
                    if a.key_index == 1 { 0 } else { a.key_index }
                })
                // +1, because we don't want to sync the latest address again
                .unwrap_or(highest_public_address_index + 1);

            let address_start_index_internal = internal_addresses
                .first()
                .map(|a| a.key_index)
                // +1, because we don't want to sync the latest address again
                .unwrap_or_else(|| highest_internal_address_index.unwrap_or(0) + 1);

            let sync_options = match &mut sync_options {
                Some(sync_options) => {
                    sync_options.force_syncing = true;
                    sync_options.address_start_index = address_start_index;
                    sync_options.address_start_index_internal = address_start_index_internal;
                    Some(sync_options.clone())
                }
                None => Some(SyncOptions {
                    force_syncing: true,
                    // skip previous addresses
                    address_start_index,
                    address_start_index_internal,
                    ..Default::default()
                }),
            };

            self.sync(sync_options).await?;

            let output_count = self.read().await.unspent_outputs.len();

            // break if we didn't find more outputs with the new addresses
            if output_count <= latest_outputs_count {
                break;
            }

            latest_outputs_count = output_count;

            // Update address_gap_limit to only generate the amount of addresses we need to have `address_gap_limit`
            // amount of empty addresses after the latest one with outputs

            let account = self.read().await;

            let highest_address_index = account
                .public_addresses
                .iter()
                .max_by_key(|a| *a.key_index())
                .map(|a| *a.key_index())
                .expect("account needs to have at least one public address");

            let highest_address_index_internal = account
                .internal_addresses
                .iter()
                .max_by_key(|a| *a.key_index())
                .map(|a| *a.key_index())
                .unwrap_or(0);

            drop(account);

            let addresses_with_unspent_outputs = self.addresses_with_unspent_outputs().await?;

            let (addresses_with_outputs_internal, address_with_outputs): (
                Vec<&AddressWithUnspentOutputs>,
                Vec<&AddressWithUnspentOutputs>,
            ) = addresses_with_unspent_outputs.iter().partition(|a| a.internal);

            let latest_address_index_with_outputs = address_with_outputs
                .iter()
                .max_by_key(|a| *a.key_index())
                .map(|a| *a.key_index() as i64)
                // -1 as default, because we will subtract this value and want to have the amount of empty addresses in
                // a row and not the address index
                .unwrap_or(-1);

            let latest_address_index_with_outputs_internal = addresses_with_outputs_internal
                .iter()
                .max_by_key(|a| *a.key_index())
                .map(|a| *a.key_index() as i64)
                // -1 as default, because we will subtract this value and want to have the amount of empty addresses in
                // a row and not the address index
                .unwrap_or(-1);

            log::debug!(
                "new highest_address_index: {highest_address_index}, internal: {highest_address_index_internal}"
            );
            log::debug!(
                "new latest_address_index_with_outputs: {latest_address_index_with_outputs:?}, internal: {latest_address_index_with_outputs_internal:?}"
            );

            let empty_addresses_in_row = (highest_address_index as i64 - latest_address_index_with_outputs) as u32;

            let empty_addresses_in_row_internal =
                (highest_address_index_internal as i64 - latest_address_index_with_outputs_internal) as u32;

            log::debug!(
                "new empty_addresses_in_row: {empty_addresses_in_row}, internal: {empty_addresses_in_row_internal}"
            );

            if empty_addresses_in_row > address_gap_limit {
                log::debug!("empty_addresses_in_row: {empty_addresses_in_row}, setting address_gap_limit to 0");
                address_gap_limit = 0;
            } else {
                address_gap_limit -= empty_addresses_in_row;
            }
            if empty_addresses_in_row_internal > address_gap_limit_internal {
                log::debug!(
                    "empty_addresses_in_row_internal: {empty_addresses_in_row_internal}, setting address_gap_limit_internal to 0"
                );
                address_gap_limit_internal = 0;
            } else {
                address_gap_limit_internal -= empty_addresses_in_row_internal;
            }

            log::debug!("new address_gap_limit: {address_gap_limit}, internal: {address_gap_limit_internal}");

            if address_gap_limit == 0 && address_gap_limit_internal == 0 {
                break;
            }
        }

        self.clean_account_after_recovery(highest_public_address_index, highest_internal_address_index)
            .await;

        #[cfg(feature = "storage")]
        {
            log::debug!(
                "[search_addresses_with_outputs] storing account {} with new synced data",
                self.alias().await
            );
            self.save(None).await?;
        }

        Ok(latest_outputs_count)
    }

    /// During search_addresses_with_outputs we created new addresses that don't have funds, so we remove them again.
    // `old_highest_public_address_index` is not optional, because we need to have at least one public address in the
    // account
    async fn clean_account_after_recovery(
        &self,
        old_highest_public_address_index: u32,
        old_highest_internal_address_index: Option<u32>,
    ) {
        let mut account = self.write().await;

        let (internal_addresses_with_unspent_outputs, public_addresses_with_spent_outputs): (
            Vec<&AddressWithUnspentOutputs>,
            Vec<&AddressWithUnspentOutputs>,
        ) = account
            .addresses_with_unspent_outputs()
            .iter()
            .partition(|address| address.internal);

        let highest_public_index_with_outputs = public_addresses_with_spent_outputs
            .iter()
            .map(|a| a.key_index)
            .max()
            // We want to have at least one public address
            .unwrap_or(0);

        let highest_internal_index_with_outputs = internal_addresses_with_unspent_outputs
            .iter()
            .map(|a| a.key_index)
            .max();

        // The new highest index should be either the old one before we searched for funds or if we found addresses with
        // funds the highest index from an address with outputs
        let new_latest_public_index = cmp::max(highest_public_index_with_outputs, old_highest_public_address_index);
        account.public_addresses = account
            .public_addresses
            .clone()
            .into_iter()
            .filter(|a| a.key_index <= new_latest_public_index)
            .collect();

        account.internal_addresses =
            if old_highest_internal_address_index.is_none() && highest_internal_index_with_outputs.is_none() {
                // For internal addresses we don't leave an empty address, that's only required for the public address
                Vec::new()
            } else {
                let new_latest_internal_index = cmp::max(
                    highest_internal_index_with_outputs.unwrap_or(0),
                    old_highest_internal_address_index.unwrap_or(0),
                );
                account
                    .internal_addresses
                    .clone()
                    .into_iter()
                    .filter(|a| a.key_index <= new_latest_internal_index)
                    .collect()
            };
    }
}

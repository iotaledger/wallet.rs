// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod output_ids;
mod outputs;

use std::collections::HashSet;

use iota_client::block::address::Address;

use crate::account::{
    handle::AccountHandle, operations::syncing::SyncOptions, types::address::AddressWithUnspentOutputs,
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

        let mut addresses_before_syncing = self.addresses().await?;

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
        } else if options.address_start_index != 0 || options.address_start_index_internal != 0 {
            // Filter addresses when address_start_index(_internal) is not 0, so we skip these addresses
            addresses_before_syncing.retain(|a| {
                if a.internal {
                    a.key_index >= options.address_start_index_internal
                } else {
                    a.key_index >= options.address_start_index
                }
            });
        }

        // Check if selected addresses contains addresses with balance so we can correctly update them
        let addresses_with_unspent_outputs = self.addresses_with_unspent_outputs().await?;
        let mut addresses_with_old_output_ids = Vec::new();
        for address in addresses_before_syncing {
            let mut output_ids = Vec::new();
            // Add currently known unspent output ids, so we can later compare them with the new output ids and see if
            // one got spent (is missing in the new returned output ids)
            if let Some(address_with_unspent_outputs) = addresses_with_unspent_outputs
                .iter()
                .find(|a| a.address == address.address)
            {
                output_ids = address_with_unspent_outputs.output_ids.to_vec();
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
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(all(feature = "events", any(feature = "ledger-nano", feature = "ledger-nano-simulator")))]
use crate::events::types::{AddressData, WalletEvent};
use crate::{
    account::{
        handle::AccountHandle,
        types::address::{AccountAddress, AddressWrapper},
    },
    client,
    signing::{GenerateAddressMetadata, Network},
};

use serde::{Deserialize, Serialize};

/// Options for address generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGenerationOptions {
    pub internal: bool,
    pub metadata: GenerateAddressMetadata,
}

impl Default for AddressGenerationOptions {
    fn default() -> Self {
        Self {
            internal: false,
            metadata: GenerateAddressMetadata {
                syncing: false,
                network: Network::Testnet,
            },
        }
    }
}

/// Generate addresses and stores them in the account
pub async fn generate_addresses(
    account_handle: &AccountHandle,
    amount: usize,
    options: AddressGenerationOptions,
) -> crate::Result<Vec<AccountAddress>> {
    log::debug!("[ADDRESS GENERATION] generating {} addresses", amount);
    let mut account = account_handle.write().await;
    let signer = crate::signing::get_signer().await;
    let mut signer = signer.lock().await;

    // get the highest index for the public or internal addresses
    let highest_current_index_plus_one = if options.internal {
        account.internal_addresses.len()
    } else {
        account.public_addresses.len()
    };

    // get bech32_hrp
    let bech32_hrp = {
        match account.public_addresses.first() {
            Some(address) => address.address.bech32_hrp.to_string(),
            // Only when we create a new account we don't have the first address and need to get the information from
            // the client Doesn't work for offline creating, should we use the network from the
            // GenerateAddressMetadata instead to use `iota` or `atoi`?
            None => {
                let client = client::get_client().await?;
                let bech32_hrp = client.get_bech32_hrp().await?;
                bech32_hrp
            }
        }
    };
    let mut generate_addresses = Vec::new();
    for address_index in highest_current_index_plus_one..highest_current_index_plus_one + amount {
        #[cfg(all(feature = "events", any(feature = "ledger-nano", feature = "ledger-nano-simulator")))]
        // If we don't sync, then we want to display the prompt on the ledger with the address. But the user needs to
        // have it visible on the computer first, so we need to generate it without the prompt first
        if !options.metadata.syncing {
            let mut changed_metadata = options.metadata.clone();
            changed_metadata.syncing = true;
            let address = signer
                .generate_address(&account, address_index, options.internal, changed_metadata)
                .await?;
            let address_wrapper = AddressWrapper::new(address, bech32_hrp.clone());
            self.event_emitter.lock().await.emit(
                account.index,
                WalletEvent::LedgerAddressGeneration(AddressData {
                    address: address_wrapper.to_bech32(),
                }),
            );
        }

        let address = signer
            .generate_address(&account, address_index, options.internal, options.metadata.clone())
            .await?;

        let address_wrapper = AddressWrapper::new(address, bech32_hrp.clone());

        generate_addresses.push(AccountAddress {
            address: address_wrapper,
            key_index: address_index,
            internal: options.internal,
            used: false,
        });
    }

    // add addresses to the account
    if options.internal {
        account.internal_addresses.extend(generate_addresses.clone());
    } else {
        account.public_addresses.extend(generate_addresses.clone());
    };

    #[cfg(feature = "storage")]
    log::debug!("[ADDRESS GENERATION] storing account {}", account.index());
    crate::storage::manager::get()
        .await?
        .lock()
        .await
        .save_account(&account)
        .await?;
    Ok(generate_addresses)
}

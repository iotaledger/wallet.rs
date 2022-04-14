// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::signing::{GenerateAddressMetadata, Network};
use serde::{Deserialize, Serialize};

use crate::account::{
    handle::AccountHandle,
    types::address::{AccountAddress, AddressWrapper},
};
#[cfg(all(feature = "events", any(feature = "ledger-nano", feature = "ledger-nano")))]
use crate::events::types::{AddressData, WalletEvent};

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

impl AccountHandle {
    /// Generate addresses and stores them in the account
    /// ```ignore
    /// let public_addresses = account_handle.generate_addresses(2, None).await?;
    /// // internal addresses are used for remainder outputs, if the RemainderValueStrategy for transfers is set to ChangeAddress
    /// let internal_addresses = account_handle
    ///     .generate_addresses(
    ///         1,
    ///         Some(AddressGenerationOptions {
    ///             internal: true,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// ```
    pub async fn generate_addresses(
        &self,
        amount: u32,
        options: Option<AddressGenerationOptions>,
    ) -> crate::Result<Vec<AccountAddress>> {
        let options = options.unwrap_or_default();
        log::debug!("[ADDRESS GENERATION] generating {} addresses", amount);
        let mut account = self.write().await;
        let mut signer = self.signer.lock().await;

        // get the highest index for the public or internal addresses
        let highest_current_index_plus_one = if options.internal {
            account.internal_addresses.len() as u32
        } else {
            account.public_addresses.len() as u32
        };

        // get bech32_hrp
        let bech32_hrp = {
            match account.public_addresses.first() {
                Some(address) => address.address.bech32_hrp.to_string(),
                // Only when we create a new account we don't have the first address and need to get the information
                // from the client Doesn't work for offline creating, should we use the network from the
                // GenerateAddressMetadata instead to use `iota` or `atoi`?
                None => {
                    let bech32_hrp = self
                        .client
                        .get_bech32_hrp()
                        .await
                        .unwrap_or_else(|_| "iota".to_string());
                    bech32_hrp
                }
            }
        };

        let address_range = highest_current_index_plus_one..highest_current_index_plus_one + amount;

        #[cfg(all(feature = "events", any(feature = "ledger-nano", feature = "ledger-nano")))]
        // If we don't sync, then we want to display the prompt on the ledger with the address. But the user needs to
        // have it visible on the computer first, so we need to generate it without the prompt first
        if !options.metadata.syncing {
            let mut changed_metadata = options.metadata.clone();
            changed_metadata.syncing = true;
            let addresses = signer
                .generate_addresses(
                    account.coin_type,
                    account.index,
                    address_range.clone(),
                    options.internal,
                    changed_metadata,
                )
                .await?;
            for address in addresses {
                let address_wrapper = AddressWrapper::new(address, bech32_hrp.clone());
                self.event_emitter.lock().await.emit(
                    account.index,
                    WalletEvent::LedgerAddressGeneration(AddressData {
                        address: address_wrapper.to_bech32(),
                    }),
                );
            }
        }

        let addresses = signer
            .generate_addresses(
                account.coin_type,
                account.index,
                address_range,
                options.internal,
                options.metadata.clone(),
            )
            .await?;

        let generate_addresses: Vec<AccountAddress> = addresses
            .into_iter()
            .enumerate()
            .map(|(index, address)| AccountAddress {
                address: AddressWrapper::new(address, bech32_hrp.clone()),
                key_index: highest_current_index_plus_one + index as u32,
                internal: options.internal,
                used: false,
            })
            .collect();

        // add addresses to the account
        if options.internal {
            account.internal_addresses.extend(generate_addresses.clone());
        } else {
            account.public_addresses.extend(generate_addresses.clone());
        };

        #[cfg(feature = "storage")]
        {
            log::debug!("[ADDRESS GENERATION] storing account {}", account.index());
            self.save(Some(&account)).await?;
        }
        Ok(generate_addresses)
    }
}

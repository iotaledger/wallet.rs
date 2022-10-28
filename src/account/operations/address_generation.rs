// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    constants::SHIMMER_TESTNET_BECH32_HRP,
    secret::{GenerateAddressOptions, SecretManage, SecretManager},
};
use serde::{Deserialize, Serialize};

use crate::account::{
    handle::AccountHandle,
    types::address::{AccountAddress, AddressWrapper},
};
#[cfg(all(feature = "events", any(feature = "ledger_nano", feature = "ledger_nano")))]
use crate::events::types::{AddressData, WalletEvent};

/// Options for address generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGenerationOptions {
    pub internal: bool,
    pub metadata: GenerateAddressOptions,
}

impl Default for AddressGenerationOptions {
    fn default() -> Self {
        Self {
            internal: false,
            metadata: GenerateAddressOptions {
                ledger_nano_prompt: false,
            },
        }
    }
}

impl AccountHandle {
    /// Generate addresses and stores them in the account
    /// ```ignore
    /// let public_addresses = account_handle.generate_addresses(2, None).await?;
    /// // internal addresses are used for remainder outputs, if the RemainderValueStrategy for transactions is set to ChangeAddress
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
        log::debug!(
            "[ADDRESS GENERATION] generating {amount} addresses, internal: {}",
            options.internal
        );
        if amount == 0 {
            return Ok(vec![]);
        }

        let account = self.read().await;

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
                // GenerateAddressOptions instead to use `iota` or `atoi`?
                None => self
                    .client
                    .get_bech32_hrp()
                    .unwrap_or_else(|_| SHIMMER_TESTNET_BECH32_HRP.to_string()),
            }
        };

        let address_range = highest_current_index_plus_one..highest_current_index_plus_one + amount;

        let addresses = match &*self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger_nano) => {
                // If we don't sync, then we want to display the prompt on the ledger with the address. But the user
                // needs to have it visible on the computer first, so we need to generate it without the
                // prompt first
                if !options.metadata.ledger_nano_prompt {
                    let mut changed_metadata = options.metadata.clone();
                    // Change metadata so ledger will not show the prompt the first time
                    changed_metadata.ledger_nano_prompt = true;
                    let mut addresses = Vec::new();

                    for address_index in address_range {
                        #[cfg(feature = "events")]
                        {
                            // Generate without prompt to be able to display it
                            let address = ledger_nano
                                .generate_addresses(
                                    account.coin_type,
                                    account.index,
                                    address_index..address_index + 1,
                                    options.internal,
                                    Some(changed_metadata.clone()),
                                )
                                .await?;
                            self.event_emitter.lock().await.emit(
                                account.index,
                                WalletEvent::LedgerAddressGeneration(AddressData {
                                    address: address[0].to_bech32(bech32_hrp.clone()),
                                }),
                            );
                        }
                        // Generate with prompt so the user can verify
                        let address = ledger_nano
                            .generate_addresses(
                                account.coin_type,
                                account.index,
                                address_index..address_index + 1,
                                options.internal,
                                Some(options.metadata.clone()),
                            )
                            .await?;
                        addresses.push(address[0]);
                    }
                    addresses
                } else {
                    ledger_nano
                        .generate_addresses(
                            account.coin_type,
                            account.index,
                            address_range.clone(),
                            options.internal,
                            Some(options.metadata),
                        )
                        .await?
                }
            }
            #[cfg(feature = "stronghold")]
            SecretManager::Stronghold(stronghold) => {
                stronghold
                    .generate_addresses(
                        account.coin_type,
                        account.index,
                        address_range,
                        options.internal,
                        Some(options.metadata.clone()),
                    )
                    .await?
            }
            SecretManager::Mnemonic(mnemonic) => {
                mnemonic
                    .generate_addresses(
                        account.coin_type,
                        account.index,
                        address_range,
                        options.internal,
                        Some(options.metadata.clone()),
                    )
                    .await?
            }
            SecretManager::Placeholder(_) => vec![],
        };

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

        drop(account);
        self.update_account_addresses(options.internal, generate_addresses.clone())
            .await?;

        Ok(generate_addresses)
    }

    /// Generate an internal address and store in the account, internal addresses are used for remainder outputs
    pub(crate) async fn generate_remainder_address(&self) -> crate::Result<AccountAddress> {
        let result = self
            .generate_addresses(
                1,
                Some(AddressGenerationOptions {
                    internal: true,
                    ..Default::default()
                }),
            )
            .await?
            .first()
            .ok_or(crate::Error::FailedToGetRemainder)?
            .clone();

        Ok(result)
    }
}

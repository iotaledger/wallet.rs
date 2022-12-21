// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::Ordering;

#[cfg(all(feature = "events", feature = "ledger_nano"))]
use crate::events::types::{AddressData, WalletEvent};
use crate::{
    account_manager::AccountManager,
    client::{
        block::address::Address,
        secret::{GenerateAddressOptions, SecretManage, SecretManager},
    },
};

impl AccountManager {
    /// Generate an address without storing it
    /// ```ignore
    /// let public_addresses = account_manager
    ///     .generate_address(
    ///         0,
    ///         false,
    ///         0,
    ///         false,
    ///     )
    ///     .await?;
    /// ```
    pub async fn generate_address(
        &self,
        account_index: u32,
        internal: bool,
        address_index: u32,
        options: Option<GenerateAddressOptions>,
    ) -> crate::Result<Address> {
        let address = match &*self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger_nano) => {
                // If we don't sync, then we want to display the prompt on the ledger with the address. But the user
                // needs to have it visible on the computer first, so we need to generate it without the
                // prompt first
                if options.as_ref().map_or(false, |o| o.ledger_nano_prompt) {
                    #[cfg(feature = "events")]
                    {
                        let changed_options = options.clone().map(|mut options| {
                            // Change options so ledger will not show the prompt the first time
                            options.ledger_nano_prompt = false;
                            options
                        });
                        // Generate without prompt to be able to display it
                        let address = ledger_nano
                            .generate_addresses(
                                self.coin_type.load(Ordering::Relaxed),
                                account_index,
                                address_index..address_index + 1,
                                internal,
                                changed_options,
                            )
                            .await?;

                        let bech32_hrp = self.get_bech32_hrp().await?;

                        self.event_emitter.lock().await.emit(
                            account_index,
                            WalletEvent::LedgerAddressGeneration(AddressData {
                                address: address[0].to_bech32(bech32_hrp),
                            }),
                        );
                    }

                    // Generate with prompt so the user can verify
                    ledger_nano
                        .generate_addresses(
                            self.coin_type.load(Ordering::Relaxed),
                            account_index,
                            address_index..address_index + 1,
                            internal,
                            options,
                        )
                        .await?
                } else {
                    ledger_nano
                        .generate_addresses(
                            self.coin_type.load(Ordering::Relaxed),
                            account_index,
                            address_index..address_index + 1,
                            internal,
                            options,
                        )
                        .await?
                }
            }
            #[cfg(feature = "stronghold")]
            SecretManager::Stronghold(stronghold) => {
                stronghold
                    .generate_addresses(
                        self.coin_type.load(Ordering::Relaxed),
                        account_index,
                        address_index..address_index + 1,
                        internal,
                        options,
                    )
                    .await?
            }
            SecretManager::Mnemonic(mnemonic) => {
                mnemonic
                    .generate_addresses(
                        self.coin_type.load(Ordering::Relaxed),
                        account_index,
                        address_index..address_index + 1,
                        internal,
                        options,
                    )
                    .await?
            }
            SecretManager::Placeholder(_) => return Err(iota_client::Error::PlaceholderSecretManager.into()),
        };

        Ok(*address.first().ok_or(crate::Error::MissingParameter("address"))?)
    }

    // Get the bech32 hrp from the first account address or if not existent, from the client
    #[allow(dead_code)]
    pub(crate) async fn get_bech32_hrp(&self) -> crate::Result<String> {
        Ok(match self.get_accounts().await?.first() {
            Some(account) => account
                .public_addresses()
                .await
                .first()
                .expect("missing first public address")
                .address
                .bech32_hrp
                .clone(),
            None => {
                self.client_options
                    .read()
                    .await
                    .clone()
                    .finish()?
                    .get_bech32_hrp()
                    .await?
            }
        })
    }
}

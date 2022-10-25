// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::Ordering;

use iota_client::{
    block::address::Address,
    constants::SHIMMER_TESTNET_BECH32_HRP,
    secret::{GenerateAddressMetadata, SecretManage, SecretManager},
};

use crate::{
    account::{handle::AccountHandle, AddressGenerationOptions},
    account_manager::{AccountManager, SyncOptions},
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
        ledger_nano_prompt: bool,
    ) -> crate::Result<Address> {
        // get bech32_hrp
        let bech32_hrp = match self.get_accounts().await?.first() {
            Some(account) => {
                account
                    .public_addresses()
                    .await
                    .first()
                    .expect("missing first public address")
                    .address
                    .bech32_hrp
            }
            None => self
                .client_options
                .read()
                .await
                .finish()?
                .get_bech32_hrp()
                .unwrap_or_else(|_| SHIMMER_TESTNET_BECH32_HRP.to_string()),
        };

        Ok(match &*self.secret_manager.read().await {
            #[cfg(feature = "ledger_nano")]
            SecretManager::LedgerNano(ledger_nano) => {
                // If we don't sync, then we want to display the prompt on the ledger with the address. But the user
                // needs to have it visible on the computer first, so we need to generate it without the
                // prompt first
                if ledger_nano_prompt {
                    #[cfg(feature = "events")]
                    {
                        // Change metadata so ledger will not show the prompt the first time
                        // Generate without prompt to be able to display it
                        let address = ledger_nano
                            .generate_addresses(
                                account.coin_type,
                                account.index,
                                address_index..address_index + 1,
                                options.internal,
                                false,
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
                            true,
                        )
                        .await?;
                    address
                } else {
                    ledger_nano
                        .generate_addresses(
                            self.coin_type,
                            account_index,
                            address_index..address_index + 1,
                            internal,
                            false,
                        )
                        .await?
                }
            }
            #[cfg(feature = "stronghold")]
            SecretManager::Stronghold(stronghold) => *stronghold
                .generate_addresses(
                    self.coin_type.load(Ordering::Relaxed),
                    account_index,
                    address_index..address_index + 1,
                    internal,
                    false,
                )
                .await?
                .first()
                .ok_or_else(|| Err(crate::Error::MissingParameter("address")))?,
            SecretManager::Mnemonic(mnemonic) => *mnemonic
                .generate_addresses(
                    self.coin_type.load(Ordering::Relaxed),
                    account_index,
                    address_index..address_index + 1,
                    internal,
                    false,
                )
                .await?
                .first()
                .ok_or_else(|| Err(crate::Error::MissingParameter("address")))?,
            SecretManager::Placeholder(_) => return Err(iota_client::Error::PlaceholderSecretManager.into()),
        })
    }
}

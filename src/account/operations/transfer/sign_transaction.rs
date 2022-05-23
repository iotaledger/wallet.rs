// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "stronghold")]
use iota_client::secret::SecretManager;
use iota_client::{
    api::{PreparedTransactionData, SignedTransactionData},
    bee_block::{address::Address, unlock::Unlocks},
    secret::SecretManageExt,
};

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

impl AccountHandle {
    /// Function to sign a transaction essence
    pub async fn sign_transaction_essence(
        &self,
        prepared_transaction_data: &PreparedTransactionData,
    ) -> crate::Result<SignedTransactionData> {
        log::debug!("[TRANSFER] sign_transaction_essence");
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            self.read().await.index,
            WalletEvent::TransferProgress(TransferProgressEvent::SigningTransaction),
        );

        // If we use stronghold we need to read the snapshot in case it hasn't been done already
        #[cfg(feature = "stronghold")]
        if let SecretManager::Stronghold(stronghold_secret_manager) = &mut *self.secret_manager.write().await {
            stronghold_secret_manager.read_stronghold_snapshot().await?;
        }

        let unlocks = self
            .secret_manager
            .read()
            .await
            .sign_transaction_essence(prepared_transaction_data)
            .await?;

        let transaction_payload =
            TransactionPayload::new(prepared_transaction_data.essence.clone(), Unlocks::new(unlocks)?)?;

        // Validate signature after signing. The hashed public key needs to match the input address
        let mut input_addresses = Vec::new();
        for input_signing_data in &prepared_transaction_data.inputs_data {
            let (_bech32_hrp, address) = Address::try_from_bech32(&input_signing_data.bech32_address)?;
            input_addresses.push(address);
        }

        log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);

        Ok(SignedTransactionData {
            transaction_payload,
            inputs_data: prepared_transaction_data.inputs_data.clone(),
        })
    }
}

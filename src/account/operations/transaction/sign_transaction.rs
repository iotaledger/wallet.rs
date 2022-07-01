// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(all(feature = "events", feature = "ledger_nano"))]
use iota_client::api::PreparedTransactionDataDto;
#[cfg(all(feature = "events", feature = "ledger_nano"))]
use iota_client::secret::ledger_nano::needs_blind_signing;
#[cfg(feature = "stronghold")]
use iota_client::secret::SecretManager;
use iota_client::{
    api::{PreparedTransactionData, SignedTransactionData},
    secret::SecretManageExt,
};

use crate::account::{handle::AccountHandle, operations::transaction::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransactionProgressEvent, WalletEvent};

impl AccountHandle {
    /// Function to sign a transaction essence
    pub async fn sign_transaction_essence(
        &self,
        prepared_transaction_data: &PreparedTransactionData,
    ) -> crate::Result<SignedTransactionData> {
        log::debug!("[TRANSACTION] sign_transaction_essence");
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            self.read().await.index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::SigningTransaction),
        );

        // If we use stronghold we need to read the snapshot in case it hasn't been done already
        #[cfg(feature = "stronghold")]
        if let SecretManager::Stronghold(stronghold_secret_manager) = &mut *self.secret_manager.write().await {
            stronghold_secret_manager.read_stronghold_snapshot().await?;
        }

        #[cfg(all(feature = "events", feature = "ledger_nano"))]
        match &*self.secret_manager.read().await {
            SecretManager::LedgerNano(ledger) | SecretManager::LedgerNanoSimulator(ledger) => {
                let ledger_status = ledger.get_ledger_status().await;
                if let Some(buffer_size) = ledger_status.buffer_size() {
                    if needs_blind_signing(prepared_transaction_data, buffer_size) {
                        self.event_emitter.lock().await.emit(
                            self.read().await.index,
                            WalletEvent::TransactionProgress(TransactionProgressEvent::PreparedTransactionEssenceHash(
                                hex::encode(prepared_transaction_data.essence.hash()),
                            )),
                        );
                    } else {
                        self.event_emitter.lock().await.emit(
                            self.read().await.index,
                            WalletEvent::TransactionProgress(TransactionProgressEvent::PreparedTransaction(Box::new(
                                PreparedTransactionDataDto::from(prepared_transaction_data),
                            ))),
                        );
                    }
                }
            }
            _ => {}
        }

        let unlocks = self
            .secret_manager
            .read()
            .await
            .sign_transaction_essence(prepared_transaction_data)
            .await?;

        let transaction_payload = TransactionPayload::new(prepared_transaction_data.essence.clone(), unlocks)?;

        log::debug!("[TRANSACTION] signed transaction: {:?}", transaction_payload);

        Ok(SignedTransactionData {
            transaction_payload,
            inputs_data: prepared_transaction_data.inputs_data.clone(),
        })
    }
}

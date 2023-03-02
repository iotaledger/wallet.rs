// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::{transaction::validate_transaction_payload_length, PreparedTransactionData, SignedTransactionData},
    secret::SecretManageExt,
};
#[cfg(all(feature = "events", feature = "ledger_nano"))]
use {
    iota_client::api::PreparedTransactionDataDto, iota_client::secret::ledger_nano::needs_blind_signing,
    iota_client::secret::SecretManager,
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
        log::debug!("[TRANSACTION] prepared_transaction_data {prepared_transaction_data:?}");
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            self.read().await.index,
            WalletEvent::TransactionProgress(TransactionProgressEvent::SigningTransaction),
        );

        #[cfg(all(feature = "events", feature = "ledger_nano"))]
        if let SecretManager::LedgerNano(ledger) = &*self.secret_manager.read().await {
            let ledger_nano_status = ledger.get_ledger_nano_status().await;
            if let Some(buffer_size) = ledger_nano_status.buffer_size() {
                if needs_blind_signing(prepared_transaction_data, buffer_size) {
                    self.event_emitter.lock().await.emit(
                        self.read().await.index,
                        WalletEvent::TransactionProgress(TransactionProgressEvent::PreparedTransactionEssenceHash(
                            prefix_hex::encode(prepared_transaction_data.essence.hash()),
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

        let unlocks = self
            .secret_manager
            .read()
            .await
            .sign_transaction_essence(prepared_transaction_data)
            .await?;

        let transaction_payload = TransactionPayload::new(prepared_transaction_data.essence.clone(), unlocks)?;

        log::debug!("[TRANSACTION] signed transaction: {:?}", transaction_payload);

        validate_transaction_payload_length(&transaction_payload)?;

        Ok(SignedTransactionData {
            transaction_payload,
            inputs_data: prepared_transaction_data.inputs_data.clone(),
        })
    }
}

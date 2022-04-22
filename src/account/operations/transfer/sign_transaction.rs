// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::verify_semantic,
    bee_message::{
        address::Address,
        output::{unlock_condition::UnlockCondition, Output},
        payload::transaction::TransactionEssence,
        unlock_block::UnlockBlocks,
    },
    constants::{IOTA_BECH32_HRP, SHIMMER_BECH32_HRP},
    secret::{types::InputSigningData, Network, SecretManager, SecretManagerType, SignMessageMetadata},
};

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

impl AccountHandle {
    /// Function to sign a transaction essence
    pub(crate) async fn sign_tx_essence(
        &self,
        essence: TransactionEssence,
        transaction_inputs: Vec<InputSigningData>,
        remainder: Option<Output>,
    ) -> crate::Result<TransactionPayload> {
        log::debug!("[TRANSFER] sign_tx_essence");
        let account = self.read().await;
        #[cfg(feature = "events")]
        self.event_emitter.lock().await.emit(
            account.index,
            WalletEvent::TransferProgress(TransferProgressEvent::SigningTransaction),
        );
        let (_remainder_deposit_address, remainder_value) = match remainder {
            Some(remainder) => {
                let mut remainder_address = None;
                if let Some(unlock_conditions) = remainder.unlock_conditions() {
                    for unlock_condition in unlock_conditions.iter() {
                        if let UnlockCondition::Address(address_unlock_condition) = unlock_condition {
                            remainder_address.replace(*address_unlock_condition.address());
                        }
                    }
                }
                (remainder_address, remainder.amount())
            }
            None => (None, 0),
        };
        let network = match account
            .public_addresses
            .first()
            .expect("Missing first public address")
            .address
            .bech32_hrp()
        {
            IOTA_BECH32_HRP | SHIMMER_BECH32_HRP => Network::Mainnet,
            _ => Network::Testnet,
        };

        // todo remainder address
        // let remainder = match remainder_deposit_address {
        //     Some(remainder_deposit_address) => Some(iota_client::secret::types::AccountAddress {
        //         address: remainder_deposit_address.address.inner,
        //         key_index: remainder_deposit_address.key_index,
        //         internal: remainder_deposit_address.internal,
        //     }),
        //     None => None,
        // };

        // If we use stronghold we need to read the snapshot in case it hasn't been done already
        #[cfg(feature = "stronghold")]
        if let SecretManagerType::Stronghold(stronghold_secret_manager) = &mut *self.secret_manager.write().await {
            stronghold_secret_manager.read_stronghold_snapshot().await?;
        }

        let unlock_blocks = self
            .secret_manager
            .read()
            .await
            .sign_transaction_essence(
                &essence,
                &transaction_inputs,
                SignMessageMetadata {
                    remainder_value,
                    // todo remainder address
                    remainder_deposit_address: None,
                    network,
                },
            )
            .await?;

        let transaction_payload = TransactionPayload::new(essence, UnlockBlocks::new(unlock_blocks)?)?;

        // Validate signature after signing. The hashed public key needs to match the input address
        let mut input_addresses = Vec::new();
        for input_signing_data in &transaction_inputs {
            let (_bech32_hrp, address) = Address::try_from_bech32(&input_signing_data.bech32_address)?;
            input_addresses.push(address);
        }

        let (local_time, milestone_index) = self.client.get_time_and_milestone_checked().await?;
        verify_semantic(&transaction_inputs, &transaction_payload, milestone_index, local_time)?;

        log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);

        Ok(transaction_payload)
    }
}

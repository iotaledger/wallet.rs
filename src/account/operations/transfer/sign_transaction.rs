// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    bee_message::{
        address::Address,
        output::{unlock_condition::UnlockCondition, Output},
        payload::transaction::TransactionEssence,
        unlock_block::UnlockBlocks,
    },
    signing::{types::InputSigningData, verify_unlock_blocks, Network, SignMessageMetadata},
};

use crate::account::{handle::AccountHandle, operations::transfer::TransactionPayload};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

impl AccountHandle {
    /// Function to sign a transaction essence
    pub(crate) async fn sign_tx_essence(
        &self,
        essence: TransactionEssence,
        mut transaction_inputs: Vec<InputSigningData>,
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
            "iota" => Network::Mainnet,
            _ => Network::Testnet,
        };

        // todo remainder address
        // let remainder = match remainder_deposit_address {
        //     Some(remainder_deposit_address) => Some(iota_client::signing::types::AccountAddress {
        //         address: remainder_deposit_address.address.inner,
        //         key_index: remainder_deposit_address.key_index,
        //         internal: remainder_deposit_address.internal,
        //     }),
        //     None => None,
        // };
        let unlock_blocks = self
            .signer
            .lock()
            .await
            .sign_transaction_essence(
                &essence,
                &mut transaction_inputs,
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
        verify_unlock_blocks(&transaction_payload, input_addresses)?;
        log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);
        Ok(transaction_payload)
    }
}

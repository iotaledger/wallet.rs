// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    handle::AccountHandle,
    operations::transfer::{Remainder, TransactionPayload},
};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    bee_message::{address::Address, payload::transaction::TransactionEssence, unlock_block::UnlockBlocks},
    signing::{types::InputSigningData, verify_unlock_blocks, Network, SignMessageMetadata},
};

/// Function to sign a transaction essence
pub(crate) async fn sign_tx_essence(
    account_handle: &AccountHandle,
    essence: TransactionEssence,
    mut transaction_inputs: Vec<InputSigningData>,
    remainder: Option<Remainder>,
) -> crate::Result<TransactionPayload> {
    log::debug!("[TRANSFER] sign_tx_essence");
    let account = account_handle.read().await;
    #[cfg(feature = "events")]
    account_handle.event_emitter.lock().await.emit(
        account.index,
        WalletEvent::TransferProgress(TransferProgressEvent::SigningTransaction),
    );
    let (remainder_deposit_address, remainder_value) = match remainder {
        Some(remainder) => (Some(remainder.address), remainder.amount),
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

    let remainder = match remainder_deposit_address {
        Some(remainder_deposit_address) => Some(iota_client::signing::types::AccountAddress {
            address: remainder_deposit_address.address.inner,
            key_index: remainder_deposit_address.key_index,
            internal: remainder_deposit_address.internal,
        }),
        None => None,
    };
    let unlock_blocks = account_handle
        .signer
        .lock()
        .await
        .sign_transaction_essence(
            &essence,
            &mut transaction_inputs,
            SignMessageMetadata {
                remainder_value,
                remainder_deposit_address: remainder.as_ref(),
                network,
            },
        )
        .await?;

    let transaction_payload = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?)
        .finish()?;

    // Validate signature after signing. The hashed public key needs to match the input address
    let mut input_addresses = Vec::new();
    for input_signing_data in &transaction_inputs {
        let address = Address::try_from_bech32(&input_signing_data.bech32_address)?;
        input_addresses.push(address);
    }
    verify_unlock_blocks(&transaction_payload, input_addresses)?;
    log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);
    Ok(transaction_payload)
}

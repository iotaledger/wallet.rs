// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};
use crate::{
    account::{
        handle::AccountHandle,
        operations::transfer::{Remainder, TransactionPayload},
    },
    signing::{SignMessageMetadata, TransactionInput},
};

use iota_client::bee_message::{
    address::Address,
    payload::transaction::Essence,
    unlock::{UnlockBlock, UnlockBlocks},
};

/// Function to sign a transaction essence
pub(crate) async fn sign_tx_essence(
    account_handle: &AccountHandle,
    essence: Essence,
    mut transaction_inputs: Vec<TransactionInput>,
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
        "iota" => crate::signing::Network::Mainnet,
        _ => crate::signing::Network::Testnet,
    };

    let unlock_blocks = crate::signing::get_signer()
        .await
        .lock()
        .await
        .sign_transaction(
            &account,
            &essence,
            &mut transaction_inputs,
            SignMessageMetadata {
                remainder_value,
                remainder_deposit_address: remainder_deposit_address.as_ref(),
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
    for input in transaction_inputs {
        if input.address_internal {
            let position = account
                .internal_addresses
                .binary_search_by_key(&(input.address_index, input.address_internal), |a| {
                    (a.key_index, a.internal)
                })
                .map_err(|e| crate::Error::InputAddressNotFound)?;
            input_addresses.push(account.internal_addresses[position].address.inner);
        } else {
            let position = account
                .public_addresses
                .binary_search_by_key(&(input.address_index, input.address_internal), |a| {
                    (a.key_index, a.internal)
                })
                .map_err(|e| crate::Error::InputAddressNotFound)?;
            input_addresses.push(account.public_addresses[position].address.inner);
        }
    }
    verify_unlock_blocks(&transaction_payload, input_addresses)?;
    log::debug!("[TRANSFER] signed transaction: {:?}", transaction_payload);
    Ok(transaction_payload)
}

fn verify_unlock_blocks(transaction_payload: &TransactionPayload, inputs: Vec<Address>) -> crate::Result<()> {
    let essence_hash = transaction_payload.essence().hash();
    let Essence::Regular(essence) = transaction_payload.essence();
    let unlock_blocks = transaction_payload.unlock_blocks();
    for (index, address) in inputs.iter().enumerate() {
        verify_signature(address, unlock_blocks, index, &essence_hash)?;
    }
    Ok(())
}

fn verify_signature(
    address: &Address,
    unlock_blocks: &UnlockBlocks,
    index: usize,
    essence_hash: &[u8; 32],
) -> crate::Result<()> {
    let signature_unlock_block = match unlock_blocks.get(index) {
        Some(unlock_block) => match unlock_block {
            UnlockBlock::Signature(b) => b,
            UnlockBlock::Reference(b) => match unlock_blocks.get(b.index().into()) {
                Some(UnlockBlock::Signature(unlock_block)) => unlock_block,
                _ => return Err(crate::Error::MissingUnlockBlock),
            },
        },
        None => return Err(crate::Error::MissingUnlockBlock),
    };
    Ok(address.verify(essence_hash, signature_unlock_block)?)
}

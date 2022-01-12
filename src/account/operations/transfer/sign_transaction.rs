// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    handle::AccountHandle,
    operations::transfer::{Remainder, TransactionPayload},
};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    bee_message::{payload::transaction::TransactionEssence, unlock_block::UnlockBlocks},
    signing::{mnemonic::IOTA_COIN_TYPE, verify_unlock_blocks, Network, SignMessageMetadata, TransactionInput},
};

/// Function to sign a transaction essence
pub(crate) async fn sign_tx_essence(
    account_handle: &AccountHandle,
    essence: TransactionEssence,
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
            IOTA_COIN_TYPE,
            account.index,
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
